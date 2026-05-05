//! Tax Registration Repository
//!
//! Storage interface for tax registration data.

use atlas_shared::{
    TaxRegistration, TaxRegistrationSummary,
    AtlasResult,
};
use async_trait::async_trait;
use uuid::Uuid;

/// Repository trait for tax registration data storage
#[async_trait]
pub trait TaxRegistrationRepository: Send + Sync {
    /// Create a new tax registration
    async fn create_registration(
        &self,
        org_id: Uuid,
        registration_number: &str,
        registration_type: &str,
        tax_purpose: &str,
        party_type: &str,
        party_id: Option<Uuid>,
        party_name: Option<&str>,
        jurisdiction_code: &str,
        country_code: &str,
        state_code: Option<&str>,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
        is_default: bool,
        reporting_name: Option<&str>,
        legal_entity_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TaxRegistration>;

    /// Get a tax registration by ID
    async fn get_registration(&self, id: Uuid) -> AtlasResult<Option<TaxRegistration>>;

    /// Get a tax registration by org + registration number
    async fn get_registration_by_number(
        &self,
        org_id: Uuid,
        registration_number: &str,
    ) -> AtlasResult<Option<TaxRegistration>>;

    /// List registrations with optional filters
    async fn list_registrations(
        &self,
        org_id: Uuid,
        party_type: Option<&str>,
        status: Option<&str>,
        jurisdiction_code: Option<&str>,
        country_code: Option<&str>,
    ) -> AtlasResult<Vec<TaxRegistration>>;

    /// Update registration status
    async fn update_registration_status(
        &self,
        id: Uuid,
        status: &str,
        effective_to: Option<chrono::NaiveDate>,
    ) -> AtlasResult<TaxRegistration>;

    /// Get registration summary counts
    async fn get_summary(&self, org_id: Uuid) -> AtlasResult<TaxRegistrationSummary>;

    /// Get the next sequential registration number for an org
    async fn get_next_registration_number(&self, org_id: Uuid) -> AtlasResult<i32>;
}

/// PostgreSQL implementation of the TaxRegistrationRepository
pub struct PostgresTaxRegistrationRepository {
    _pool: sqlx::PgPool,
}

impl PostgresTaxRegistrationRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { _pool: pool }
    }
}

#[async_trait]
impl TaxRegistrationRepository for PostgresTaxRegistrationRepository {
    async fn create_registration(
        &self,
        org_id: Uuid,
        registration_number: &str,
        registration_type: &str,
        tax_purpose: &str,
        party_type: &str,
        party_id: Option<Uuid>,
        party_name: Option<&str>,
        jurisdiction_code: &str,
        country_code: &str,
        state_code: Option<&str>,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
        is_default: bool,
        reporting_name: Option<&str>,
        legal_entity_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TaxRegistration> {
        // Stub: in production, this would INSERT into PostgreSQL
        Ok(TaxRegistration {
            id: Uuid::new_v4(),
            organization_id: org_id,
            registration_number: registration_number.to_string(),
            registration_type: registration_type.to_string(),
            tax_purpose: tax_purpose.to_string(),
            party_type: party_type.to_string(),
            party_id,
            party_name: party_name.map(|s| s.to_string()),
            jurisdiction_code: jurisdiction_code.to_string(),
            country_code: country_code.to_string(),
            state_code: state_code.map(|s| s.to_string()),
            status: "active".to_string(),
            effective_from,
            effective_to,
            is_default,
            reporting_name: reporting_name.map(|s| s.to_string()),
            legal_entity_id,
            validation_status: "pending".to_string(),
            last_validated_at: None,
            source: "manual".to_string(),
            created_by,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    async fn get_registration(&self, _id: Uuid) -> AtlasResult<Option<TaxRegistration>> {
        Ok(None)
    }

    async fn get_registration_by_number(
        &self,
        _org_id: Uuid,
        _registration_number: &str,
    ) -> AtlasResult<Option<TaxRegistration>> {
        Ok(None)
    }

    async fn list_registrations(
        &self,
        _org_id: Uuid,
        _party_type: Option<&str>,
        _status: Option<&str>,
        _jurisdiction_code: Option<&str>,
        _country_code: Option<&str>,
    ) -> AtlasResult<Vec<TaxRegistration>> {
        Ok(vec![])
    }

    async fn update_registration_status(
        &self,
        _id: Uuid,
        _status: &str,
        _effective_to: Option<chrono::NaiveDate>,
    ) -> AtlasResult<TaxRegistration> {
        Err(atlas_shared::AtlasError::EntityNotFound("Not implemented".to_string()))
    }

    async fn get_summary(&self, org_id: Uuid) -> AtlasResult<TaxRegistrationSummary> {
        Ok(TaxRegistrationSummary {
            organization_id: org_id,
            total_registrations: 0,
            active_registrations: 0,
            suspended_registrations: 0,
            expired_registrations: 0,
            pending_registrations: 0,
            first_party_count: 0,
            third_party_count: 0,
            jurisdictions_covered: 0,
        })
    }

    async fn get_next_registration_number(&self, _org_id: Uuid) -> AtlasResult<i32> {
        Ok(1)
    }
}
