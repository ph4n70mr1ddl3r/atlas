//! Recruiting Management Repository
//!
//! PostgreSQL storage for recruiting data.

use atlas_shared::{
    JobRequisition, Candidate, JobApplication, Interview, JobOffer,
    RecruitingDashboard, AtlasResult, AtlasError,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

/// Repository trait for Recruiting data storage
#[async_trait]
pub trait RecruitingRepository: Send + Sync {
    // Requisitions
    async fn create_requisition(
        &self, org_id: Uuid, requisition_number: &str, title: &str, description: Option<&str>,
        department: Option<&str>, location: Option<&str>, employment_type: &str,
        position_type: &str, vacancies: i32, priority: &str,
        salary_min: Option<&str>, salary_max: Option<&str>, currency: Option<&str>,
        required_skills: Option<&serde_json::Value>, qualifications: Option<&str>,
        experience_years_min: Option<i32>, experience_years_max: Option<i32>,
        education_level: Option<&str>, hiring_manager_id: Option<Uuid>,
        recruiter_id: Option<Uuid>, target_start_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JobRequisition>;
    async fn get_requisition(&self, id: Uuid) -> AtlasResult<Option<JobRequisition>>;
    async fn get_requisition_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<JobRequisition>>;
    async fn list_requisitions(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<JobRequisition>>;
    async fn update_requisition_status(&self, id: Uuid, status: &str) -> AtlasResult<JobRequisition>;
    async fn delete_requisition(&self, org_id: Uuid, number: &str) -> AtlasResult<()>;

    // Candidates
    async fn create_candidate(
        &self, org_id: Uuid, first_name: &str, last_name: &str,
        email: Option<&str>, phone: Option<&str>, address: Option<&str>,
        city: Option<&str>, state: Option<&str>, country: Option<&str>,
        postal_code: Option<&str>, linkedin_url: Option<&str>,
        source: Option<&str>, source_detail: Option<&str>, resume_url: Option<&str>,
        current_employer: Option<&str>, current_title: Option<&str>,
        years_of_experience: Option<i32>, education_level: Option<&str>,
        skills: Option<&serde_json::Value>, notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Candidate>;
    async fn get_candidate(&self, id: Uuid) -> AtlasResult<Option<Candidate>>;
    async fn list_candidates(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<Candidate>>;
    async fn update_candidate_status(&self, id: Uuid, status: &str) -> AtlasResult<Candidate>;
    async fn delete_candidate(&self, id: Uuid) -> AtlasResult<()>;

    // Applications
    async fn create_application(
        &self, org_id: Uuid, requisition_id: Uuid, candidate_id: Uuid,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JobApplication>;
    async fn get_application(&self, id: Uuid) -> AtlasResult<Option<JobApplication>>;
    async fn list_applications(
        &self, org_id: Uuid, requisition_id: Option<Uuid>,
        candidate_id: Option<Uuid>, status: Option<&str>,
    ) -> AtlasResult<Vec<JobApplication>>;
    async fn update_application_status(
        &self, id: Uuid, status: &str, notes: Option<&str>,
    ) -> AtlasResult<JobApplication>;

    // Interviews
    async fn create_interview(
        &self, org_id: Uuid, application_id: Uuid, interview_type: &str,
        round: i32, scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
        duration_minutes: i32, location: Option<&str>, meeting_link: Option<&str>,
        interviewer_ids: Option<&serde_json::Value>,
        interviewer_names: Option<&serde_json::Value>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<Interview>;
    async fn get_interview(&self, id: Uuid) -> AtlasResult<Option<Interview>>;
    async fn list_interviews(&self, application_id: Uuid) -> AtlasResult<Vec<Interview>>;
    async fn update_interview_status(&self, id: Uuid, status: &str) -> AtlasResult<Interview>;
    async fn complete_interview(
        &self, id: Uuid, feedback: Option<&str>, rating: Option<i32>,
        recommendation: Option<&str>,
    ) -> AtlasResult<Interview>;
    async fn delete_interview(&self, id: Uuid) -> AtlasResult<()>;

    // Offers
    async fn create_offer(
        &self, org_id: Uuid, application_id: Uuid, offer_number: Option<&str>,
        job_title: &str, department: Option<&str>, location: Option<&str>,
        employment_type: &str, start_date: Option<chrono::NaiveDate>,
        salary_offered: Option<&str>, salary_currency: Option<&str>,
        salary_frequency: Option<&str>, signing_bonus: Option<&str>,
        benefits_summary: Option<&str>, terms_and_conditions: Option<&str>,
        response_deadline: Option<chrono::DateTime<chrono::Utc>>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JobOffer>;
    async fn get_offer(&self, id: Uuid) -> AtlasResult<Option<JobOffer>>;
    async fn list_offers(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<JobOffer>>;
    async fn update_offer_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<JobOffer>;
    async fn decline_offer(&self, id: Uuid, notes: Option<&str>) -> AtlasResult<JobOffer>;
    async fn delete_offer(&self, id: Uuid) -> AtlasResult<()>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<RecruitingDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresRecruitingRepository {
    pool: PgPool,
}

impl PostgresRecruitingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn get_num(row: &sqlx::postgres::PgRow, col: &str) -> String {
    let v: f64 = row.try_get(col).unwrap_or(0.0);
    format!("{:.2}", v)
}

fn get_opt_num(row: &sqlx::postgres::PgRow, col: &str) -> Option<String> {
    match row.try_get::<Option<f64>, _>(col) {
        Ok(Some(v)) => Some(format!("{:.2}", v)),
        _ => None,
    }
}

fn row_to_requisition(row: &sqlx::postgres::PgRow) -> JobRequisition {
    JobRequisition {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        requisition_number: row.get("requisition_number"),
        title: row.get("title"),
        description: row.get("description"),
        department: row.get("department"),
        location: row.get("location"),
        employment_type: row.get("employment_type"),
        position_type: row.get("position_type"),
        vacancies: row.get("vacancies"),
        priority: row.get("priority"),
        salary_min: get_opt_num(row, "salary_min"),
        salary_max: get_opt_num(row, "salary_max"),
        currency: row.try_get("currency").unwrap_or_else(|_| "USD".to_string()),
        required_skills: row.get("required_skills"),
        qualifications: row.get("qualifications"),
        experience_years_min: row.get("experience_years_min"),
        experience_years_max: row.get("experience_years_max"),
        education_level: row.get("education_level"),
        hiring_manager_id: row.get("hiring_manager_id"),
        recruiter_id: row.get("recruiter_id"),
        target_start_date: row.get("target_start_date"),
        status: row.get("status"),
        posted_date: row.get("posted_date"),
        closed_date: row.get("closed_date"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_candidate(row: &sqlx::postgres::PgRow) -> Candidate {
    Candidate {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        candidate_number: row.get("candidate_number"),
        first_name: row.get("first_name"),
        last_name: row.get("last_name"),
        email: row.get("email"),
        phone: row.get("phone"),
        address: row.get("address"),
        city: row.get("city"),
        state: row.get("state"),
        country: row.get("country"),
        postal_code: row.get("postal_code"),
        linkedin_url: row.get("linkedin_url"),
        source: row.get("source"),
        source_detail: row.get("source_detail"),
        resume_url: row.get("resume_url"),
        cover_letter_url: row.get("cover_letter_url"),
        current_employer: row.get("current_employer"),
        current_title: row.get("current_title"),
        years_of_experience: row.get("years_of_experience"),
        education_level: row.get("education_level"),
        skills: row.get("skills"),
        notes: row.get("notes"),
        status: row.get("status"),
        tags: row.get("tags"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_application(row: &sqlx::postgres::PgRow) -> JobApplication {
    JobApplication {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        application_number: row.get("application_number"),
        requisition_id: row.get("requisition_id"),
        candidate_id: row.get("candidate_id"),
        status: row.get("status"),
        match_score: get_num(row, "match_score"),
        screening_notes: row.get("screening_notes"),
        rejection_reason: row.get("rejection_reason"),
        applied_at: row.get("applied_at"),
        last_status_change: row.get("last_status_change"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_interview(row: &sqlx::postgres::PgRow) -> Interview {
    Interview {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        application_id: row.get("application_id"),
        interview_type: row.get("interview_type"),
        round: row.get("round"),
        scheduled_at: row.get("scheduled_at"),
        duration_minutes: row.get("duration_minutes"),
        location: row.get("location"),
        meeting_link: row.get("meeting_link"),
        interviewer_ids: row.get("interviewer_ids"),
        interviewer_names: row.get("interviewer_names"),
        status: row.get("status"),
        feedback: row.get("feedback"),
        rating: row.get("rating"),
        recommendation: row.get("recommendation"),
        completed_at: row.get("completed_at"),
        notes: row.get("notes"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

fn row_to_offer(row: &sqlx::postgres::PgRow) -> JobOffer {
    JobOffer {
        id: row.get("id"),
        organization_id: row.get("organization_id"),
        application_id: row.get("application_id"),
        offer_number: row.get("offer_number"),
        job_title: row.get("job_title"),
        department: row.get("department"),
        location: row.get("location"),
        employment_type: row.get("employment_type"),
        start_date: row.get("start_date"),
        salary_offered: get_opt_num(row, "salary_offered"),
        salary_currency: row.try_get("salary_currency").unwrap_or_else(|_| "USD".to_string()),
        salary_frequency: row.try_get("salary_frequency").unwrap_or_else(|_| "annual".to_string()),
        signing_bonus: get_opt_num(row, "signing_bonus"),
        benefits_summary: row.get("benefits_summary"),
        terms_and_conditions: row.get("terms_and_conditions"),
        status: row.get("status"),
        offer_date: row.get("offer_date"),
        response_deadline: row.get("response_deadline"),
        responded_at: row.get("responded_at"),
        response_notes: row.get("response_notes"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
        metadata: row.get("metadata"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}

#[async_trait]
impl RecruitingRepository for PostgresRecruitingRepository {
    // ========================================================================
    // Requisitions
    // ========================================================================

    async fn create_requisition(
        &self, org_id: Uuid, requisition_number: &str, title: &str, description: Option<&str>,
        department: Option<&str>, location: Option<&str>, employment_type: &str,
        position_type: &str, vacancies: i32, priority: &str,
        salary_min: Option<&str>, salary_max: Option<&str>, currency: Option<&str>,
        required_skills: Option<&serde_json::Value>, qualifications: Option<&str>,
        experience_years_min: Option<i32>, experience_years_max: Option<i32>,
        education_level: Option<&str>, hiring_manager_id: Option<Uuid>,
        recruiter_id: Option<Uuid>, target_start_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JobRequisition> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.job_requisitions
                (organization_id, requisition_number, title, description,
                 department, location, employment_type, position_type, vacancies, priority,
                 salary_min, salary_max, currency, required_skills, qualifications,
                 experience_years_min, experience_years_max, education_level,
                 hiring_manager_id, recruiter_id, target_start_date, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22)
               RETURNING *"#,
        )
        .bind(org_id).bind(requisition_number).bind(title).bind(description)
        .bind(department).bind(location).bind(employment_type).bind(position_type)
        .bind(vacancies).bind(priority)
        .bind(salary_min.map(|v| v.parse::<f64>().unwrap_or(0.0)))
        .bind(salary_max.map(|v| v.parse::<f64>().unwrap_or(0.0)))
        .bind(currency.unwrap_or("USD"))
        .bind(required_skills.unwrap_or(&serde_json::json!([])))
        .bind(qualifications)
        .bind(experience_years_min).bind(experience_years_max)
        .bind(education_level)
        .bind(hiring_manager_id).bind(recruiter_id)
        .bind(target_start_date).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_requisition(&row))
    }

    async fn get_requisition(&self, id: Uuid) -> AtlasResult<Option<JobRequisition>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.job_requisitions WHERE id = $1",
        )
        .bind(id).fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_requisition(&r)))
    }

    async fn get_requisition_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<JobRequisition>> {
        let row = sqlx::query(
            "SELECT * FROM _atlas.job_requisitions WHERE organization_id = $1 AND requisition_number = $2",
        )
        .bind(org_id).bind(number).fetch_optional(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_requisition(&r)))
    }

    async fn list_requisitions(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<JobRequisition>> {
        let rows = if let Some(s) = status {
            sqlx::query(
                "SELECT * FROM _atlas.job_requisitions WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC",
            ).bind(org_id).bind(s).fetch_all(&self.pool).await
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.job_requisitions WHERE organization_id = $1 ORDER BY created_at DESC",
            ).bind(org_id).fetch_all(&self.pool).await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_requisition).collect())
    }

    async fn update_requisition_status(&self, id: Uuid, status: &str) -> AtlasResult<JobRequisition> {
        let row = sqlx::query(
            r#"UPDATE _atlas.job_requisitions
               SET status = $2,
                   posted_date = CASE WHEN $2 = 'open' AND posted_date IS NULL THEN now() ELSE posted_date END,
                   closed_date = CASE WHEN $2 IN ('closed','cancelled','filled') AND closed_date IS NULL THEN now() ELSE closed_date END,
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(status).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_requisition(&row))
    }

    async fn delete_requisition(&self, org_id: Uuid, number: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.job_requisitions WHERE organization_id = $1 AND requisition_number = $2")
            .bind(org_id).bind(number).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Candidates
    // ========================================================================

    async fn create_candidate(
        &self, org_id: Uuid, first_name: &str, last_name: &str,
        email: Option<&str>, phone: Option<&str>, address: Option<&str>,
        city: Option<&str>, state: Option<&str>, country: Option<&str>,
        postal_code: Option<&str>, linkedin_url: Option<&str>,
        source: Option<&str>, source_detail: Option<&str>, resume_url: Option<&str>,
        current_employer: Option<&str>, current_title: Option<&str>,
        years_of_experience: Option<i32>, education_level: Option<&str>,
        skills: Option<&serde_json::Value>, notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Candidate> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.candidates
                (organization_id, first_name, last_name, email, phone, address,
                 city, state, country, postal_code, linkedin_url, source, source_detail,
                 resume_url, current_employer, current_title, years_of_experience,
                 education_level, skills, notes, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21)
               RETURNING *"#,
        )
        .bind(org_id).bind(first_name).bind(last_name).bind(email)
        .bind(phone).bind(address).bind(city).bind(state)
        .bind(country).bind(postal_code).bind(linkedin_url)
        .bind(source).bind(source_detail).bind(resume_url)
        .bind(current_employer).bind(current_title).bind(years_of_experience)
        .bind(education_level).bind(skills.unwrap_or(&serde_json::json!([])))
        .bind(notes).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_candidate(&row))
    }

    async fn get_candidate(&self, id: Uuid) -> AtlasResult<Option<Candidate>> {
        let row = sqlx::query("SELECT * FROM _atlas.candidates WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_candidate(&r)))
    }

    async fn list_candidates(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<Candidate>> {
        let rows = if let Some(s) = status {
            sqlx::query(
                "SELECT * FROM _atlas.candidates WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC",
            ).bind(org_id).bind(s).fetch_all(&self.pool).await
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.candidates WHERE organization_id = $1 ORDER BY created_at DESC",
            ).bind(org_id).fetch_all(&self.pool).await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_candidate).collect())
    }

    async fn update_candidate_status(&self, id: Uuid, status: &str) -> AtlasResult<Candidate> {
        let row = sqlx::query(
            "UPDATE _atlas.candidates SET status = $2, updated_at = now() WHERE id = $1 RETURNING *",
        )
        .bind(id).bind(status).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_candidate(&row))
    }

    async fn delete_candidate(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.candidates WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Applications
    // ========================================================================

    async fn create_application(
        &self, org_id: Uuid, requisition_id: Uuid, candidate_id: Uuid,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JobApplication> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.job_applications
                (organization_id, requisition_id, candidate_id, created_by)
               VALUES ($1, $2, $3, $4) RETURNING *"#,
        )
        .bind(org_id).bind(requisition_id).bind(candidate_id).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_application(&row))
    }

    async fn get_application(&self, id: Uuid) -> AtlasResult<Option<JobApplication>> {
        let row = sqlx::query("SELECT * FROM _atlas.job_applications WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_application(&r)))
    }

    async fn list_applications(
        &self, org_id: Uuid, requisition_id: Option<Uuid>,
        candidate_id: Option<Uuid>, status: Option<&str>,
    ) -> AtlasResult<Vec<JobApplication>> {
        let mut query_str = "SELECT * FROM _atlas.job_applications WHERE organization_id = $1".to_string();
        let mut param_idx = 2;
        let has_req = requisition_id.is_some();
        let has_cand = candidate_id.is_some();
        let has_status = status.is_some();

        if has_req { query_str.push_str(&format!(" AND requisition_id = ${}", param_idx)); param_idx += 1; }
        if has_cand { query_str.push_str(&format!(" AND candidate_id = ${}", param_idx)); param_idx += 1; }
        if has_status { query_str.push_str(&format!(" AND status = ${}", param_idx)); param_idx += 1; }
        query_str.push_str(" ORDER BY applied_at DESC");

        let mut query = sqlx::query(&query_str).bind(org_id);
        if let Some(rid) = requisition_id { query = query.bind(rid); }
        if let Some(cid) = candidate_id { query = query.bind(cid); }
        if let Some(s) = status { query = query.bind(s); }

        let rows = query.fetch_all(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_application).collect())
    }

    async fn update_application_status(
        &self, id: Uuid, status: &str, notes: Option<&str>,
    ) -> AtlasResult<JobApplication> {
        let row = sqlx::query(
            r#"UPDATE _atlas.job_applications
               SET status = $2,
                   last_status_change = now(),
                   rejection_reason = CASE WHEN $2 = 'rejected' THEN COALESCE($3, rejection_reason) ELSE rejection_reason END,
                   screening_notes = CASE WHEN $2 IN ('screening','interview','assessment') THEN COALESCE($3, screening_notes) ELSE screening_notes END,
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(notes)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_application(&row))
    }

    // ========================================================================
    // Interviews
    // ========================================================================

    async fn create_interview(
        &self, org_id: Uuid, application_id: Uuid, interview_type: &str,
        round: i32, scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
        duration_minutes: i32, location: Option<&str>, meeting_link: Option<&str>,
        interviewer_ids: Option<&serde_json::Value>,
        interviewer_names: Option<&serde_json::Value>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<Interview> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.interviews
                (organization_id, application_id, interview_type, round,
                 scheduled_at, duration_minutes, location, meeting_link,
                 interviewer_ids, interviewer_names, notes, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12) RETURNING *"#,
        )
        .bind(org_id).bind(application_id).bind(interview_type).bind(round)
        .bind(scheduled_at).bind(duration_minutes).bind(location).bind(meeting_link)
        .bind(interviewer_ids.unwrap_or(&serde_json::json!([])))
        .bind(interviewer_names.unwrap_or(&serde_json::json!([])))
        .bind(notes).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_interview(&row))
    }

    async fn get_interview(&self, id: Uuid) -> AtlasResult<Option<Interview>> {
        let row = sqlx::query("SELECT * FROM _atlas.interviews WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_interview(&r)))
    }

    async fn list_interviews(&self, application_id: Uuid) -> AtlasResult<Vec<Interview>> {
        let rows = sqlx::query(
            "SELECT * FROM _atlas.interviews WHERE application_id = $1 ORDER BY round, scheduled_at",
        )
        .bind(application_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_interview).collect())
    }

    async fn update_interview_status(&self, id: Uuid, status: &str) -> AtlasResult<Interview> {
        let row = sqlx::query(
            "UPDATE _atlas.interviews SET status = $2, updated_at = now() WHERE id = $1 RETURNING *",
        )
        .bind(id).bind(status).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_interview(&row))
    }

    async fn complete_interview(
        &self, id: Uuid, feedback: Option<&str>, rating: Option<i32>,
        recommendation: Option<&str>,
    ) -> AtlasResult<Interview> {
        let row = sqlx::query(
            r#"UPDATE _atlas.interviews
               SET status = 'completed', feedback = COALESCE($2, feedback),
                   rating = $3, recommendation = $4,
                   completed_at = now(), updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(feedback).bind(rating).bind(recommendation)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_interview(&row))
    }

    async fn delete_interview(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.interviews WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Offers
    // ========================================================================

    async fn create_offer(
        &self, org_id: Uuid, application_id: Uuid, offer_number: Option<&str>,
        job_title: &str, department: Option<&str>, location: Option<&str>,
        employment_type: &str, start_date: Option<chrono::NaiveDate>,
        salary_offered: Option<&str>, salary_currency: Option<&str>,
        salary_frequency: Option<&str>, signing_bonus: Option<&str>,
        benefits_summary: Option<&str>, terms_and_conditions: Option<&str>,
        response_deadline: Option<chrono::DateTime<chrono::Utc>>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<JobOffer> {
        let row = sqlx::query(
            r#"INSERT INTO _atlas.job_offers
                (organization_id, application_id, offer_number, job_title,
                 department, location, employment_type, start_date,
                 salary_offered, salary_currency, salary_frequency, signing_bonus,
                 benefits_summary, terms_and_conditions, response_deadline, created_by)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16) RETURNING *"#,
        )
        .bind(org_id).bind(application_id).bind(offer_number).bind(job_title)
        .bind(department).bind(location).bind(employment_type).bind(start_date)
        .bind(salary_offered.map(|v| v.parse::<f64>().unwrap_or(0.0)))
        .bind(salary_currency.unwrap_or("USD"))
        .bind(salary_frequency.unwrap_or("annual"))
        .bind(signing_bonus.map(|v| v.parse::<f64>().unwrap_or(0.0)))
        .bind(benefits_summary).bind(terms_and_conditions)
        .bind(response_deadline).bind(created_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_offer(&row))
    }

    async fn get_offer(&self, id: Uuid) -> AtlasResult<Option<JobOffer>> {
        let row = sqlx::query("SELECT * FROM _atlas.job_offers WHERE id = $1")
            .bind(id).fetch_optional(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row.map(|r| row_to_offer(&r)))
    }

    async fn list_offers(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<JobOffer>> {
        let rows = if let Some(s) = status {
            sqlx::query(
                "SELECT * FROM _atlas.job_offers WHERE organization_id = $1 AND status = $2 ORDER BY created_at DESC",
            ).bind(org_id).bind(s).fetch_all(&self.pool).await
        } else {
            sqlx::query(
                "SELECT * FROM _atlas.job_offers WHERE organization_id = $1 ORDER BY created_at DESC",
            ).bind(org_id).fetch_all(&self.pool).await
        }.map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(rows.iter().map(row_to_offer).collect())
    }

    async fn update_offer_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<JobOffer> {
        let row = sqlx::query(
            r#"UPDATE _atlas.job_offers
               SET status = $2,
                   approved_by = CASE WHEN $2 = 'approved' THEN $3 ELSE approved_by END,
                   approved_at = CASE WHEN $2 = 'approved' THEN now() ELSE approved_at END,
                   offer_date = CASE WHEN $2 = 'extended' THEN now() ELSE offer_date END,
                   responded_at = CASE WHEN $2 IN ('accepted','declined') THEN now() ELSE responded_at END,
                   updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(status).bind(approved_by)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_offer(&row))
    }

    async fn decline_offer(&self, id: Uuid, notes: Option<&str>) -> AtlasResult<JobOffer> {
        let row = sqlx::query(
            r#"UPDATE _atlas.job_offers
               SET status = 'declined', responded_at = now(),
                   response_notes = $2, updated_at = now()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id).bind(notes)
        .fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(row_to_offer(&row))
    }

    async fn delete_offer(&self, id: Uuid) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.job_offers WHERE id = $1")
            .bind(id).execute(&self.pool).await
            .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<RecruitingDashboard> {
        let req_row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'open') as open_count,
                COUNT(*) FILTER (WHERE status IN ('filled','closed') AND updated_at >= date_trunc('month', now())) as filled_month
               FROM _atlas.job_requisitions WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let cand_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _atlas.candidates WHERE organization_id = $1",
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let app_row = sqlx::query(
            r#"SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE applied_at >= date_trunc('month', now())) as month_count
               FROM _atlas.job_applications WHERE organization_id = $1"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let interview_month: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM _atlas.interviews
               WHERE organization_id = $1 AND completed_at >= date_trunc('month', now())"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let offers_pending: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM _atlas.job_offers
               WHERE organization_id = $1 AND status IN ('draft','pending_approval','approved','extended')"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let hires_month: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM _atlas.job_applications
               WHERE organization_id = $1 AND status = 'hired' AND last_status_change >= date_trunc('month', now())"#,
        )
        .bind(org_id).fetch_one(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        // Requisitions by status
        let req_status_rows = sqlx::query(
            r#"SELECT status, COUNT(*) as cnt FROM _atlas.job_requisitions
               WHERE organization_id = $1 GROUP BY status ORDER BY cnt DESC"#,
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let requisitions_by_status: serde_json::Value = req_status_rows.iter().map(|r| {
            serde_json::json!({
                "status": r.get::<String, _>("status"),
                "count": r.get::<i64, _>("cnt"),
            })
        }).collect();

        // Applications by status
        let app_status_rows = sqlx::query(
            r#"SELECT status, COUNT(*) as cnt FROM _atlas.job_applications
               WHERE organization_id = $1 GROUP BY status ORDER BY cnt DESC"#,
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let applications_by_status: serde_json::Value = app_status_rows.iter().map(|r| {
            serde_json::json!({
                "status": r.get::<String, _>("status"),
                "count": r.get::<i64, _>("cnt"),
            })
        }).collect();

        // Top departments
        let dept_rows = sqlx::query(
            r#"SELECT department, COUNT(*) as cnt FROM _atlas.job_requisitions
               WHERE organization_id = $1 AND department IS NOT NULL
               GROUP BY department ORDER BY cnt DESC LIMIT 10"#,
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let top_departments: serde_json::Value = dept_rows.iter().map(|r| {
            serde_json::json!({
                "department": r.get::<String, _>("department"),
                "count": r.get::<i64, _>("cnt"),
            })
        }).collect();

        // Recent applications
        let recent_rows = sqlx::query(
            r#"SELECT a.id, a.application_number, a.status, a.applied_at,
                      c.first_name, c.last_name, r.title as requisition_title
               FROM _atlas.job_applications a
               LEFT JOIN _atlas.candidates c ON a.candidate_id = c.id
               LEFT JOIN _atlas.job_requisitions r ON a.requisition_id = r.id
               WHERE a.organization_id = $1 ORDER BY a.applied_at DESC LIMIT 10"#,
        )
        .bind(org_id).fetch_all(&self.pool).await
        .map_err(|e| AtlasError::DatabaseError(e.to_string()))?;

        let recent_applications: serde_json::Value = recent_rows.iter().map(|r| {
            serde_json::json!({
                "id": r.get::<Uuid, _>("id").to_string(),
                "applicationNumber": r.get::<Option<String>, _>("application_number").unwrap_or_default(),
                "status": r.get::<String, _>("status"),
                "candidateName": format!("{} {}", r.get::<Option<String>, _>("first_name").unwrap_or_default(), r.get::<Option<String>, _>("last_name").unwrap_or_default()),
                "requisitionTitle": r.get::<Option<String>, _>("requisition_title").unwrap_or_default(),
                "appliedAt": r.get::<chrono::DateTime<chrono::Utc>, _>("applied_at").to_rfc3339(),
            })
        }).collect();

        Ok(RecruitingDashboard {
            total_requisitions: req_row.get::<i64, _>("total") as i32,
            open_requisitions: req_row.get::<i64, _>("open_count") as i32,
            total_candidates: cand_count as i32,
            total_applications: app_row.get::<i64, _>("total") as i32,
            applications_this_month: app_row.get::<i64, _>("month_count") as i32,
            interviews_this_month: interview_month as i32,
            offers_pending: offers_pending as i32,
            hires_this_month: hires_month as i32,
            requisitions_by_status,
            applications_by_status,
            top_departments,
            recent_applications,
        })
    }
}
