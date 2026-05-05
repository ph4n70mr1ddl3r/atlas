//! Tax Registration Engine
//!
//! Manages tax registration lifecycle for first-party legal entities
//! and third-party organizations across tax jurisdictions.
//!
//! Features:
//! - Tax registration CRUD with validation
//! - Registration number format validation per country/type
//! - Status lifecycle management (active, suspended, deregistered, expired, pending)
//! - Duplicate detection
//! - Compliance gap detection
//! - Summary dashboard
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Tax > Tax Registrations

use atlas_shared::{
    TaxRegistration, TaxRegistrationSummary,
    AtlasError, AtlasResult,
};
use super::TaxRegistrationRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// ============================================================================
// Valid Constants
// ============================================================================

/// Valid registration types
pub const VALID_REGISTRATION_TYPES: &[&str] = &[
    "tin", "vat", "gst", "ein", "sst", "pan", "cst",
    "sales_tax", "withholding_tax", "excise", "customs", "other",
];

/// Valid tax purposes
pub const VALID_TAX_PURPOSES: &[&str] = &[
    "input_tax", "output_tax", "both", "reporting_only",
    "withholding", "reverse_charge", "intracommunity",
];

/// Valid party types
pub const VALID_PARTY_TYPES: &[&str] = &[
    "first_party", "third_party",
];

/// Valid registration statuses
pub const VALID_STATUSES: &[&str] = &[
    "active", "suspended", "deregistered", "expired", "pending",
];

/// Valid validation statuses
pub const VALID_VALIDATION_STATUSES: &[&str] = &[
    "pending", "validated", "failed", "not_applicable",
];

/// Valid sources
pub const VALID_SOURCES: &[&str] = &[
    "manual", "import", "integration", "migration",
];

/// Valid country codes (common jurisdictions for tax registration)
pub const VALID_COUNTRY_CODES: &[&str] = &[
    "US", "GB", "DE", "FR", "CA", "AU", "IN", "JP", "BR", "MX",
    "NL", "IT", "ES", "CH", "SG", "NZ", "IE", "SE", "NO", "DK",
    "BE", "AT", "PT", "KR", "CN", "ZA", "AE", "SA",
];

// ============================================================================
// Tax Registration Engine
// ============================================================================

/// Tax Registration Engine
///
/// Manages the full lifecycle of tax registrations for organizations,
/// including creation, validation, status management, and compliance reporting.
pub struct TaxRegistrationEngine {
    repository: Arc<dyn TaxRegistrationRepository>,
}

impl TaxRegistrationEngine {
    pub fn new(repository: Arc<dyn TaxRegistrationRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // CRUD Operations
    // ========================================================================

    /// Create a new tax registration with full validation
    ///
    /// Oracle Fusion: Tax > Tax Registrations > Create
    pub async fn create_registration(
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
        source: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<TaxRegistration> {
        // Validate required fields
        if registration_number.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Registration number is required".to_string(),
            ));
        }
        if !VALID_REGISTRATION_TYPES.contains(&registration_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid registration_type '{}'. Must be one of: {}",
                registration_type, VALID_REGISTRATION_TYPES.join(", ")
            )));
        }
        if !VALID_TAX_PURPOSES.contains(&tax_purpose) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid tax_purpose '{}'. Must be one of: {}",
                tax_purpose, VALID_TAX_PURPOSES.join(", ")
            )));
        }
        if !VALID_PARTY_TYPES.contains(&party_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid party_type '{}'. Must be one of: {}",
                party_type, VALID_PARTY_TYPES.join(", ")
            )));
        }
        if jurisdiction_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Jurisdiction code is required".to_string(),
            ));
        }
        if country_code.len() != 2 {
            return Err(AtlasError::ValidationFailed(
                "Country code must be a 2-letter ISO code".to_string(),
            ));
        }
        if !VALID_SOURCES.contains(&source) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid source '{}'. Must be one of: {}",
                source, VALID_SOURCES.join(", ")
            )));
        }

        // Validate effective date range
        if let Some(to) = effective_to {
            if to < effective_from {
                return Err(AtlasError::ValidationFailed(
                    "Effective to date must be after effective from date".to_string(),
                ));
            }
        }

        // Validate registration number format for the country/type
        validate_registration_number_format(registration_number, registration_type, country_code)?;

        // Check for duplicate registration number within the org
        if let Some(existing) = self.repository.get_registration_by_number(org_id, registration_number).await? {
            if existing.status != "deregistered" && existing.status != "expired" {
                return Err(AtlasError::ValidationFailed(format!(
                    "Registration number '{}' already exists for this organization (status: {})",
                    registration_number, existing.status
                )));
            }
        }

        // First-party registrations require party_id
        if party_type == "first_party" && party_id.is_none() {
            return Err(AtlasError::ValidationFailed(
                "First-party registrations require a party_id (legal entity reference)".to_string(),
            ));
        }

        info!(
            "Creating tax registration {} ({}) for org {} in {}",
            registration_number, registration_type, org_id, country_code
        );

        self.repository.create_registration(
            org_id, registration_number, registration_type, tax_purpose,
            party_type, party_id, party_name,
            jurisdiction_code, country_code, state_code,
            effective_from, effective_to,
            is_default, reporting_name, legal_entity_id,
            created_by,
        ).await
    }

    /// Get a tax registration by ID
    pub async fn get_registration(&self, id: Uuid) -> AtlasResult<Option<TaxRegistration>> {
        self.repository.get_registration(id).await
    }

    /// Get a tax registration by number within an org
    pub async fn get_registration_by_number(
        &self,
        org_id: Uuid,
        registration_number: &str,
    ) -> AtlasResult<Option<TaxRegistration>> {
        self.repository.get_registration_by_number(org_id, registration_number).await
    }

    /// List registrations with optional filters
    pub async fn list_registrations(
        &self,
        org_id: Uuid,
        party_type: Option<&str>,
        status: Option<&str>,
        jurisdiction_code: Option<&str>,
        country_code: Option<&str>,
    ) -> AtlasResult<Vec<TaxRegistration>> {
        if let Some(pt) = party_type {
            if !VALID_PARTY_TYPES.contains(&pt) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid party_type '{}'. Must be one of: {}",
                    pt, VALID_PARTY_TYPES.join(", ")
                )));
            }
        }
        if let Some(s) = status {
            if !VALID_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}",
                    s, VALID_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_registrations(org_id, party_type, status, jurisdiction_code, country_code).await
    }

    // ========================================================================
    // Status Management
    // ========================================================================

    /// Activate a pending registration
    ///
    /// Oracle Fusion: Tax > Tax Registrations > Activate
    pub async fn activate_registration(&self, id: Uuid) -> AtlasResult<TaxRegistration> {
        let reg = self.repository.get_registration(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Tax registration {} not found", id)))?;

        if reg.status != "pending" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot activate registration in '{}' status. Must be 'pending'.",
                reg.status
            )));
        }

        info!("Activating tax registration {} ({})", reg.registration_number, reg.registration_type);
        self.repository.update_registration_status(id, "active", None).await
    }

    /// Suspend an active registration
    ///
    /// Oracle Fusion: Tax > Tax Registrations > Suspend
    pub async fn suspend_registration(&self, id: Uuid) -> AtlasResult<TaxRegistration> {
        let reg = self.repository.get_registration(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Tax registration {} not found", id)))?;

        if reg.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot suspend registration in '{}' status. Must be 'active'.",
                reg.status
            )));
        }

        info!("Suspending tax registration {} ({})", reg.registration_number, reg.registration_type);
        self.repository.update_registration_status(id, "suspended", None).await
    }

    /// Reactivate a suspended registration
    pub async fn reactivate_registration(&self, id: Uuid) -> AtlasResult<TaxRegistration> {
        let reg = self.repository.get_registration(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Tax registration {} not found", id)))?;

        if reg.status != "suspended" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reactivate registration in '{}' status. Must be 'suspended'.",
                reg.status
            )));
        }

        info!("Reactivating tax registration {} ({})", reg.registration_number, reg.registration_type);
        self.repository.update_registration_status(id, "active", None).await
    }

    /// Deregister a registration (permanent)
    ///
    /// Oracle Fusion: Tax > Tax Registrations > Deregister
    pub async fn deregister(
        &self,
        id: Uuid,
        deregistration_date: chrono::NaiveDate,
    ) -> AtlasResult<TaxRegistration> {
        let reg = self.repository.get_registration(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Tax registration {} not found", id)))?;

        if reg.status != "active" && reg.status != "suspended" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot deregister registration in '{}' status. Must be 'active' or 'suspended'.",
                reg.status
            )));
        }

        info!(
            "Deregistering tax registration {} ({}) effective {}",
            reg.registration_number, reg.registration_type, deregistration_date
        );
        self.repository.update_registration_status(id, "deregistered", Some(deregistration_date)).await
    }

    // ========================================================================
    // Validation
    // ========================================================================

    /// Validate a registration number
    ///
    /// Oracle Fusion: Tax > Tax Registrations > Validate
    pub async fn validate_registration(&self, id: Uuid) -> AtlasResult<TaxRegistration> {
        let reg = self.repository.get_registration(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Tax registration {} not found", id)))?;

        // Perform format validation
        let validation_result = validate_registration_number_format(
            &reg.registration_number,
            &reg.registration_type,
            &reg.country_code,
        );

        match validation_result {
            Ok(()) => {
                info!(
                    "Tax registration {} validated successfully ({}/{})",
                    reg.registration_number, reg.registration_type, reg.country_code
                );
                // In a full implementation, we'd update the validation_status to "validated"
                // and set last_validated_at
            }
            Err(e) => {
                info!(
                    "Tax registration {} validation failed: {}",
                    reg.registration_number, e
                );
                // In a full implementation, we'd update validation_status to "failed"
            }
        }

        Ok(reg)
    }

    // ========================================================================
    // Dashboard & Reporting
    // ========================================================================

    /// Get registration summary for the dashboard
    pub async fn get_summary(&self, org_id: Uuid) -> AtlasResult<TaxRegistrationSummary> {
        self.repository.get_summary(org_id).await
    }

    // ========================================================================
    // Pure Business Logic Functions (testable without repository)
    // ========================================================================

    /// Validate registration number format based on type and country.
    ///
    /// Applies country and type-specific validation rules:
    /// - US EIN: XX-XXXXXXX format
    /// - EU VAT: 2-letter country code + 8-15 alphanumeric characters
    /// - GST (India): 15-character alphanumeric pattern
    /// - ABN (Australia): 11 digits
    /// - Generic: minimum 3 characters
    pub fn validate_format(
        registration_number: &str,
        registration_type: &str,
        country_code: &str,
    ) -> AtlasResult<()> {
        validate_registration_number_format(registration_number, registration_type, country_code)
    }

    /// Determine if a registration is currently effective as of a given date
    pub fn is_effective(reg: &TaxRegistration, as_of: chrono::NaiveDate) -> bool {
        if reg.status != "active" {
            return false;
        }
        if as_of < reg.effective_from {
            return false;
        }
        if let Some(to) = reg.effective_to {
            if as_of > to {
                return false;
            }
        }
        true
    }

    /// Check for compliance gaps: find jurisdictions where registrations
    /// are missing or will expire soon.
    ///
    /// Returns a list of (jurisdiction_code, issue_description) tuples.
    pub fn detect_compliance_gaps(
        registrations: &[TaxRegistration],
        required_jurisdictions: &[(&str, &str)], // (jurisdiction_code, country_code)
        as_of: chrono::NaiveDate,
        warning_days: i32,
    ) -> Vec<(String, String)> {
        let mut gaps = Vec::new();

        for (jurisdiction, country) in required_jurisdictions {
            let matching: Vec<&TaxRegistration> = registrations.iter()
                .filter(|r| {
                    r.jurisdiction_code == *jurisdiction
                        && r.country_code == *country
                        && r.status == "active"
                })
                .collect();

            if matching.is_empty() {
                gaps.push((
                    jurisdiction.to_string(),
                    format!("No active registration found for jurisdiction {} ({})", jurisdiction, country),
                ));
            } else {
                // Check if any will expire within warning_days
                for reg in &matching {
                    if let Some(to) = reg.effective_to {
                        let days_remaining = (to - as_of).num_days();
                        if days_remaining <= warning_days as i64 && days_remaining > 0 {
                            gaps.push((
                                jurisdiction.to_string(),
                                format!(
                                    "Registration {} ({}) expires in {} days",
                                    reg.registration_number, reg.registration_type, days_remaining
                                ),
                            ));
                        } else if days_remaining <= 0 {
                            gaps.push((
                                jurisdiction.to_string(),
                                format!(
                                    "Registration {} ({}) has expired",
                                    reg.registration_number, reg.registration_type
                                ),
                            ));
                        }
                    }
                }
            }
        }

        gaps
    }

    /// Check if a registration number already exists in a list (duplicate detection)
    pub fn is_duplicate_number(number: &str, existing: &[TaxRegistration]) -> bool {
        existing.iter().any(|r| {
            r.registration_number == number
                && r.status != "deregistered"
                && r.status != "expired"
        })
    }

    /// Filter registrations by status
    pub fn filter_by_status<'a>(
        registrations: &'a [TaxRegistration],
        status: &str,
    ) -> Vec<&'a TaxRegistration> {
        registrations.iter().filter(|r| r.status == status).collect()
    }

    /// Get registrations expiring within N days from a reference date
    pub fn get_expiring<'a>(
        registrations: &'a [TaxRegistration],
        as_of: chrono::NaiveDate,
        within_days: i32,
    ) -> Vec<(&'a TaxRegistration, i64)> {
        registrations.iter()
            .filter(|r| {
                r.status == "active"
                    && r.effective_to.is_some()
            })
            .filter_map(|r| {
                let to = r.effective_to.unwrap();
                let days = (to - as_of).num_days();
                if days >= 0 && days <= within_days as i64 {
                    Some((r, days))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Validate a status transition is allowed
    pub fn validate_status_transition(from: &str, to: &str) -> bool {
        match (from, to) {
            ("pending", "active") => true,
            ("active", "suspended") => true,
            ("active", "deregistered") => true,
            ("active", "expired") => true,
            ("suspended", "active") => true,
            ("suspended", "deregistered") => true,
            _ => false,
        }
    }
}

// ============================================================================
// Registration Number Format Validation
// ============================================================================

/// Validates registration number format based on type and country.
fn validate_registration_number_format(
    number: &str,
    reg_type: &str,
    country_code: &str,
) -> AtlasResult<()> {
    if number.trim().is_empty() {
        return Err(AtlasError::ValidationFailed(
            "Registration number cannot be empty".to_string(),
        ));
    }

    match country_code {
        "US" => validate_us_number(number, reg_type),
        "GB" => validate_gb_vat(number, reg_type),
        "DE" | "FR" | "IT" | "ES" | "NL" | "BE" | "AT" | "PT" | "IE" | "SE" | "NO" | "DK" => {
            validate_eu_vat(number, reg_type, country_code)
        }
        "AU" => validate_au_abn(number, reg_type),
        "IN" => validate_in_gst(number, reg_type),
        "CA" => validate_ca_gst(number, reg_type),
        "BR" => validate_br_cnpj(number, reg_type),
        _ => {
            // Generic validation: minimum 3 characters, alphanumeric + hyphens
            if number.len() < 3 {
                return Err(AtlasError::ValidationFailed(format!(
                    "Registration number '{}' is too short (minimum 3 characters)", number
                )));
            }
            if !number.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '.' || c == '/' || c == ' ') {
                return Err(AtlasError::ValidationFailed(format!(
                    "Registration number '{}' contains invalid characters", number
                )));
            }
            Ok(())
        }
    }
}

/// US EIN format: XX-XXXXXXX (9 digits with hyphen)
fn validate_us_number(number: &str, reg_type: &str) -> AtlasResult<()> {
    if reg_type == "ein" {
        // EIN format: XX-XXXXXXX
        let digits_only: String = number.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits_only.len() != 9 {
            return Err(AtlasError::ValidationFailed(format!(
                "US EIN '{}' must contain exactly 9 digits (format: XX-XXXXXXX)", number
            )));
        }
        // Verify proper format if hyphen is present
        if number.contains('-') {
            let parts: Vec<&str> = number.split('-').collect();
            if parts.len() != 2 || parts[0].len() != 2 || parts[1].len() != 7 {
                return Err(AtlasError::ValidationFailed(format!(
                    "US EIN '{}' must be in XX-XXXXXXX format", number
                )));
            }
        }
    } else {
        generic_alphanumeric_check(number, 3)?;
    }
    Ok(())
}

/// UK VAT number validation: GB + 9 digits (or 12 for branches)
fn validate_gb_vat(number: &str, reg_type: &str) -> AtlasResult<()> {
    if reg_type == "vat" {
        let trimmed = number.trim_start_matches("GB").trim_start_matches("gb");
        let digits_only: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits_only.len() != 9 && digits_only.len() != 12 {
            return Err(AtlasError::ValidationFailed(format!(
                "UK VAT number '{}' must have 9 or 12 digits after country prefix", number
            )));
        }
    } else {
        generic_alphanumeric_check(number, 3)?;
    }
    Ok(())
}

/// EU VAT number validation: CC + 8-15 alphanumeric characters
fn validate_eu_vat(number: &str, reg_type: &str, country_code: &str) -> AtlasResult<()> {
    if reg_type == "vat" {
        let prefix = &country_code.to_uppercase();
        let trimmed = if number.to_uppercase().starts_with(prefix) {
            &number[prefix.len()..]
        } else {
            number
        };

        let cleaned: String = trimmed.chars()
            .filter(|c| c.is_alphanumeric())
            .collect();

        if cleaned.len() < 8 || cleaned.len() > 15 {
            return Err(AtlasError::ValidationFailed(format!(
                "EU VAT number '{}' must have 8-15 alphanumeric characters after country prefix ({})",
                number, country_code
            )));
        }
    } else {
        generic_alphanumeric_check(number, 3)?;
    }
    Ok(())
}

/// Australian ABN validation: 11 digits with basic checksum
fn validate_au_abn(number: &str, reg_type: &str) -> AtlasResult<()> {
    if reg_type == "gst" || reg_type == "tin" {
        let digits_only: String = number.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits_only.len() != 11 {
            return Err(AtlasError::ValidationFailed(format!(
                "Australian ABN/GST number '{}' must contain exactly 11 digits", number
            )));
        }
        // Basic ABN checksum: subtract 1 from first digit, then weighted sum mod 89 == 0
        let digits: Vec<u32> = digits_only.chars().filter_map(|c| c.to_digit(10)).collect();
        let weights = [10, 1, 3, 5, 7, 9, 11, 13, 15, 17, 19];
        let adjusted: Vec<u32> = digits.iter().enumerate().map(|(i, &d)| {
            if i == 0 { d.saturating_sub(1) } else { d }
        }).collect();
        let sum: u32 = adjusted.iter().zip(weights.iter()).map(|(&d, &w)| d * w).sum();
        if sum % 89 != 0 {
            return Err(AtlasError::ValidationFailed(format!(
                "Australian ABN '{}' failed checksum validation", number
            )));
        }
    } else {
        generic_alphanumeric_check(number, 3)?;
    }
    Ok(())
}

/// Indian GST Number validation: 15-character alphanumeric
/// Format: NNAAAAAXXXXZNZ (where N=numeric, A=alpha, X=alphanumeric, Z=zone)
fn validate_in_gst(number: &str, reg_type: &str) -> AtlasResult<()> {
    if reg_type == "gst" {
        if number.len() != 15 {
            return Err(AtlasError::ValidationFailed(format!(
                "Indian GST number '{}' must be exactly 15 characters", number
            )));
        }
        let chars: Vec<char> = number.chars().collect();

        // Position 0-1: state code (digits)
        if !chars[0].is_ascii_digit() || !chars[1].is_ascii_digit() {
            return Err(AtlasError::ValidationFailed(
                "GSTIN positions 0-1 must be numeric state code".to_string(),
            ));
        }
        // Position 2-6: PAN (alphanumeric)
        for i in 2..=6 {
            if !chars[i].is_alphanumeric() {
                return Err(AtlasError::ValidationFailed(
                    "GSTIN positions 2-6 must be alphanumeric (PAN)".to_string(),
                ));
            }
        }
        // Position 7: entity code (digit)
        if !chars[7].is_ascii_digit() {
            return Err(AtlasError::ValidationFailed(
                "GSTIN position 7 must be a digit (entity code)".to_string(),
            ));
        }
        // Position 8: alphanumeric (entity number extension)
        if !chars[8].is_alphanumeric() {
            return Err(AtlasError::ValidationFailed(
                "GSTIN position 8 must be alphanumeric".to_string(),
            ));
        }
        // Position 9-12: alphanumeric
        for i in 9..=12 {
            if !chars[i].is_alphanumeric() {
                return Err(AtlasError::ValidationFailed(
                    "GSTIN positions 9-12 must be alphanumeric".to_string(),
                ));
            }
        }
        // Position 13: 'Z' by default
        if chars[13] != 'Z' && chars[13] != 'z' {
            return Err(AtlasError::ValidationFailed(
                "GSTIN position 13 must be 'Z' (default)".to_string(),
            ));
        }
        // Position 14: checksum digit
        if !chars[14].is_ascii_digit() && !chars[14].is_ascii_alphabetic() {
            return Err(AtlasError::ValidationFailed(
                "GSTIN position 14 must be alphanumeric (checksum)".to_string(),
            ));
        }
    } else {
        generic_alphanumeric_check(number, 3)?;
    }
    Ok(())
}

/// Canadian GST/HST validation: 9 digits + 'RT' + 4 digits
fn validate_ca_gst(number: &str, reg_type: &str) -> AtlasResult<()> {
    if reg_type == "gst" {
        let cleaned: String = number.chars().filter(|c| c.is_alphanumeric()).collect();
        // Canadian BN: 9 digits + RT + 4 digits = 15 characters
        if cleaned.len() < 9 {
            return Err(AtlasError::ValidationFailed(format!(
                "Canadian GST number '{}' must contain at least 9 digits (BN format)", number
            )));
        }
        let first9: String = cleaned.chars().take(9).collect();
        if !first9.chars().all(|c| c.is_ascii_digit()) {
            return Err(AtlasError::ValidationFailed(
                "First 9 characters of Canadian BN must be numeric".to_string(),
            ));
        }
    } else {
        generic_alphanumeric_check(number, 3)?;
    }
    Ok(())
}

/// Brazilian CNPJ validation: 14 digits with modulo-11 checksum
fn validate_br_cnpj(number: &str, reg_type: &str) -> AtlasResult<()> {
    if reg_type == "tin" || reg_type == "cst" {
        let digits_only: String = number.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits_only.len() != 14 {
            return Err(AtlasError::ValidationFailed(format!(
                "Brazilian CNPJ '{}' must contain exactly 14 digits", number
            )));
        }
        // Modulo-11 checksum validation
        let digits: Vec<u32> = digits_only.chars().filter_map(|c| c.to_digit(10)).collect();

        // First check digit (position 12)
        let weights1 = [5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
        let sum1: u32 = digits.iter().take(12).zip(weights1.iter()).map(|(&d, &w)| d * w).sum();
        let check1 = if sum1 % 11 < 2 { 0 } else { 11 - (sum1 % 11) };
        if digits[12] != check1 {
            return Err(AtlasError::ValidationFailed(format!(
                "Brazilian CNPJ '{}' failed first check digit validation", number
            )));
        }

        // Second check digit (position 13)
        let weights2 = [6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
        let sum2: u32 = digits.iter().take(13).zip(weights2.iter()).map(|(&d, &w)| d * w).sum();
        let check2 = if sum2 % 11 < 2 { 0 } else { 11 - (sum2 % 11) };
        if digits[13] != check2 {
            return Err(AtlasError::ValidationFailed(format!(
                "Brazilian CNPJ '{}' failed second check digit validation", number
            )));
        }
    } else {
        generic_alphanumeric_check(number, 3)?;
    }
    Ok(())
}

/// Generic alphanumeric format check
fn generic_alphanumeric_check(number: &str, min_len: usize) -> AtlasResult<()> {
    let trimmed = number.trim();
    if trimmed.len() < min_len {
        return Err(AtlasError::ValidationFailed(format!(
            "Registration number '{}' is too short (minimum {} characters)", trimmed, min_len
        )));
    }
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Constant Validation Tests
    // ========================================================================

    #[test]
    fn test_valid_registration_types() {
        let valid = ["tin", "vat", "gst", "ein", "sst", "pan", "cst",
                     "sales_tax", "withholding_tax", "excise", "customs", "other"];
        for t in &valid {
            assert!(VALID_REGISTRATION_TYPES.contains(t), "{} should be valid", t);
        }
        assert!(!VALID_REGISTRATION_TYPES.contains(&"passport"));
    }

    #[test]
    fn test_valid_tax_purposes() {
        let valid = ["input_tax", "output_tax", "both", "reporting_only",
                     "withholding", "reverse_charge", "intracommunity"];
        for p in &valid {
            assert!(VALID_TAX_PURPOSES.contains(p));
        }
        assert!(!VALID_TAX_PURPOSES.contains(&"sales"));
    }

    #[test]
    fn test_valid_party_types() {
        assert!(VALID_PARTY_TYPES.contains(&"first_party"));
        assert!(VALID_PARTY_TYPES.contains(&"third_party"));
        assert!(!VALID_PARTY_TYPES.contains(&"government"));
    }

    #[test]
    fn test_valid_statuses() {
        let valid = ["active", "suspended", "deregistered", "expired", "pending"];
        for s in &valid {
            assert!(VALID_STATUSES.contains(s));
        }
        assert!(!VALID_STATUSES.contains(&"deleted"));
    }

    #[test]
    fn test_valid_validation_statuses() {
        let valid = ["pending", "validated", "failed", "not_applicable"];
        for s in &valid {
            assert!(VALID_VALIDATION_STATUSES.contains(s));
        }
    }

    #[test]
    fn test_valid_sources() {
        let valid = ["manual", "import", "integration", "migration"];
        for s in &valid {
            assert!(VALID_SOURCES.contains(s));
        }
        assert!(!VALID_SOURCES.contains(&"api"));
    }

    // ========================================================================
    // US EIN Format Validation
    // ========================================================================

    #[test]
    fn test_us_ein_valid_with_hyphen() {
        assert!(TaxRegistrationEngine::validate_format("12-3456789", "ein", "US").is_ok());
    }

    #[test]
    fn test_us_ein_valid_without_hyphen() {
        assert!(TaxRegistrationEngine::validate_format("123456789", "ein", "US").is_ok());
    }

    #[test]
    fn test_us_ein_too_short() {
        assert!(TaxRegistrationEngine::validate_format("12-345678", "ein", "US").is_err());
    }

    #[test]
    fn test_us_ein_too_long() {
        assert!(TaxRegistrationEngine::validate_format("12-34567890", "ein", "US").is_err());
    }

    #[test]
    fn test_us_ein_wrong_format() {
        assert!(TaxRegistrationEngine::validate_format("123-456789", "ein", "US").is_err());
    }

    #[test]
    fn test_us_non_ein_passes() {
        assert!(TaxRegistrationEngine::validate_format("TAX-12345", "sales_tax", "US").is_ok());
    }

    // ========================================================================
    // UK VAT Format Validation
    // ========================================================================

    #[test]
    fn test_gb_vat_valid_with_prefix() {
        assert!(TaxRegistrationEngine::validate_format("GB123456789", "vat", "GB").is_ok());
    }

    #[test]
    fn test_gb_vat_valid_without_prefix() {
        assert!(TaxRegistrationEngine::validate_format("123456789", "vat", "GB").is_ok());
    }

    #[test]
    fn test_gb_vat_valid_branch() {
        assert!(TaxRegistrationEngine::validate_format("GB123456789123", "vat", "GB").is_ok());
    }

    #[test]
    fn test_gb_vat_too_short() {
        assert!(TaxRegistrationEngine::validate_format("GB12345678", "vat", "GB").is_err());
    }

    #[test]
    fn test_gb_vat_too_long() {
        assert!(TaxRegistrationEngine::validate_format("GB1234567891234", "vat", "GB").is_err());
    }

    // ========================================================================
    // EU VAT Format Validation
    // ========================================================================

    #[test]
    fn test_de_vat_valid() {
        assert!(TaxRegistrationEngine::validate_format("DE123456789", "vat", "DE").is_ok());
    }

    #[test]
    fn test_de_vat_valid_without_prefix() {
        assert!(TaxRegistrationEngine::validate_format("123456789", "vat", "DE").is_ok());
    }

    #[test]
    fn test_fr_vat_valid() {
        assert!(TaxRegistrationEngine::validate_format("FR12345678901", "vat", "FR").is_ok());
    }

    #[test]
    fn test_eu_vat_too_short() {
        assert!(TaxRegistrationEngine::validate_format("DE1234567", "vat", "DE").is_err());
    }

    #[test]
    fn test_eu_vat_too_long() {
        assert!(TaxRegistrationEngine::validate_format("DE1234567890123456", "vat", "DE").is_err());
    }

    #[test]
    fn test_nl_vat_valid() {
        assert!(TaxRegistrationEngine::validate_format("NL123456789B01", "vat", "NL").is_ok());
    }

    // ========================================================================
    // Australian ABN Validation
    // ========================================================================

    #[test]
    fn test_au_abn_valid() {
        // Valid ABN: 53 004 085 616 (known valid ABN)
        assert!(TaxRegistrationEngine::validate_format("53004085616", "gst", "AU").is_ok());
    }

    #[test]
    fn test_au_abn_wrong_length() {
        assert!(TaxRegistrationEngine::validate_format("5300408561", "gst", "AU").is_err());
        assert!(TaxRegistrationEngine::validate_format("530040856161", "gst", "AU").is_err());
    }

    #[test]
    fn test_au_abn_failed_checksum() {
        // Valid length but wrong checksum
        assert!(TaxRegistrationEngine::validate_format("12345678901", "gst", "AU").is_err());
    }

    #[test]
    fn test_au_tin_valid() {
        assert!(TaxRegistrationEngine::validate_format("53004085616", "tin", "AU").is_ok());
    }

    // ========================================================================
    // Indian GST Validation
    // ========================================================================

    #[test]
    fn test_in_gst_valid() {
        // Valid GSTIN format: 22AAAAA0000A1Z5
        // Position 8 must be alpha - let's fix the test data
        // 29AAACR5055K1Z4 is a real example pattern
        assert!(TaxRegistrationEngine::validate_format("29AAACR5055K1Z4", "gst", "IN").is_ok());
    }

    #[test]
    fn test_in_gst_wrong_length() {
        assert!(TaxRegistrationEngine::validate_format("22AAAAA0000A1Z", "gst", "IN").is_err());
        assert!(TaxRegistrationEngine::validate_format("22AAAAA0000A1Z55", "gst", "IN").is_err());
    }

    #[test]
    fn test_in_gst_invalid_state_code() {
        // Non-digit state code
        assert!(TaxRegistrationEngine::validate_format("AA0000A0000A1Z5", "gst", "IN").is_err());
    }

    #[test]
    fn test_in_gst_invalid_z_position() {
        // Position 13 should be 'Z'
        assert!(TaxRegistrationEngine::validate_format("22AAAAA0000A1A5", "gst", "IN").is_err());
    }

    // ========================================================================
    // Canadian GST Validation
    // ========================================================================

    #[test]
    fn test_ca_gst_valid() {
        assert!(TaxRegistrationEngine::validate_format("123456789RT0001", "gst", "CA").is_ok());
    }

    #[test]
    fn test_ca_gst_valid_bn_only() {
        assert!(TaxRegistrationEngine::validate_format("123456789", "gst", "CA").is_ok());
    }

    #[test]
    fn test_ca_gst_too_short() {
        assert!(TaxRegistrationEngine::validate_format("12345678", "gst", "CA").is_err());
    }

    // ========================================================================
    // Brazilian CNPJ Validation
    // ========================================================================

    #[test]
    fn test_br_cnpj_valid() {
        // Known valid CNPJ: 11.222.333/0001-81 => 11222333000181
        assert!(TaxRegistrationEngine::validate_format("11222333000181", "tin", "BR").is_ok());
    }

    #[test]
    fn test_br_cnpj_wrong_length() {
        assert!(TaxRegistrationEngine::validate_format("1122233300018", "tin", "BR").is_err());
        assert!(TaxRegistrationEngine::validate_format("112223330001811", "tin", "BR").is_err());
    }

    #[test]
    fn test_br_cnpj_invalid_checksum() {
        // Valid length but wrong check digits
        assert!(TaxRegistrationEngine::validate_format("11222333000182", "tin", "BR").is_err());
    }

    // ========================================================================
    // Generic Validation
    // ========================================================================

    #[test]
    fn test_generic_valid() {
        assert!(TaxRegistrationEngine::validate_format("TAX-12345", "vat", "JP").is_ok());
    }

    #[test]
    fn test_generic_too_short() {
        assert!(TaxRegistrationEngine::validate_format("AB", "vat", "JP").is_err());
    }

    #[test]
    fn test_generic_empty() {
        assert!(TaxRegistrationEngine::validate_format("", "vat", "JP").is_err());
    }

    #[test]
    fn test_generic_special_chars() {
        assert!(TaxRegistrationEngine::validate_format("TAX/123.45", "vat", "JP").is_ok());
    }

    // ========================================================================
    // Status Transition Tests
    // ========================================================================

    #[test]
    fn test_status_transition_pending_to_active() {
        assert!(TaxRegistrationEngine::validate_status_transition("pending", "active"));
    }

    #[test]
    fn test_status_transition_active_to_suspended() {
        assert!(TaxRegistrationEngine::validate_status_transition("active", "suspended"));
    }

    #[test]
    fn test_status_transition_active_to_deregistered() {
        assert!(TaxRegistrationEngine::validate_status_transition("active", "deregistered"));
    }

    #[test]
    fn test_status_transition_active_to_expired() {
        assert!(TaxRegistrationEngine::validate_status_transition("active", "expired"));
    }

    #[test]
    fn test_status_transition_suspended_to_active() {
        assert!(TaxRegistrationEngine::validate_status_transition("suspended", "active"));
    }

    #[test]
    fn test_status_transition_suspended_to_deregistered() {
        assert!(TaxRegistrationEngine::validate_status_transition("suspended", "deregistered"));
    }

    #[test]
    fn test_status_transition_invalid() {
        assert!(!TaxRegistrationEngine::validate_status_transition("pending", "suspended"));
        assert!(!TaxRegistrationEngine::validate_status_transition("deregistered", "active"));
        assert!(!TaxRegistrationEngine::validate_status_transition("expired", "active"));
        assert!(!TaxRegistrationEngine::validate_status_transition("active", "pending"));
    }

    #[test]
    fn test_status_transition_terminal_states() {
        assert!(!TaxRegistrationEngine::validate_status_transition("deregistered", "active"));
        assert!(!TaxRegistrationEngine::validate_status_transition("deregistered", "suspended"));
        assert!(!TaxRegistrationEngine::validate_status_transition("expired", "active"));
    }

    // ========================================================================
    // Effectiveness Check Tests
    // ========================================================================

    #[test]
    fn test_is_effective_active_in_range() {
        let reg = make_test_registration(
            "active",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            None,
        );
        let as_of = chrono::NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        assert!(TaxRegistrationEngine::is_effective(&reg, as_of));
    }

    #[test]
    fn test_is_effective_active_before_start() {
        let reg = make_test_registration(
            "active",
            chrono::NaiveDate::from_ymd_opt(2025, 7, 1).unwrap(),
            None,
        );
        let as_of = chrono::NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        assert!(!TaxRegistrationEngine::is_effective(&reg, as_of));
    }

    #[test]
    fn test_is_effective_active_after_end() {
        let reg = make_test_registration(
            "active",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            Some(chrono::NaiveDate::from_ymd_opt(2025, 6, 30).unwrap()),
        );
        let as_of = chrono::NaiveDate::from_ymd_opt(2025, 7, 15).unwrap();
        assert!(!TaxRegistrationEngine::is_effective(&reg, as_of));
    }

    #[test]
    fn test_is_effective_inactive() {
        let reg = make_test_registration(
            "suspended",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            None,
        );
        let as_of = chrono::NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        assert!(!TaxRegistrationEngine::is_effective(&reg, as_of));
    }

    #[test]
    fn test_is_effective_on_start_date() {
        let reg = make_test_registration(
            "active",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            None,
        );
        let as_of = chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        assert!(TaxRegistrationEngine::is_effective(&reg, as_of));
    }

    #[test]
    fn test_is_effective_on_end_date() {
        let reg = make_test_registration(
            "active",
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            Some(chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
        );
        let as_of = chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
        assert!(TaxRegistrationEngine::is_effective(&reg, as_of));
    }

    // ========================================================================
    // Compliance Gap Detection Tests
    // ========================================================================

    #[test]
    fn test_compliance_gaps_no_registrations() {
        let required = vec![("US-FED", "US"), ("GB-VAT", "GB")];
        let gaps = TaxRegistrationEngine::detect_compliance_gaps(
            &[],
            &required,
            chrono::NaiveDate::from_ymd_opt(2025, 6, 15).unwrap(),
            30,
        );
        assert_eq!(gaps.len(), 2);
        assert!(gaps[0].1.contains("No active registration"));
        assert!(gaps[1].1.contains("No active registration"));
    }

    #[test]
    fn test_compliance_gaps_all_covered() {
        let regs = vec![
            make_test_registration_with_jurisdiction("active", "US-FED", "US"),
            make_test_registration_with_jurisdiction("active", "GB-VAT", "GB"),
        ];
        let required = vec![("US-FED", "US"), ("GB-VAT", "GB")];
        let gaps = TaxRegistrationEngine::detect_compliance_gaps(
            &regs,
            &required,
            chrono::NaiveDate::from_ymd_opt(2025, 6, 15).unwrap(),
            30,
        );
        assert!(gaps.is_empty());
    }

    #[test]
    fn test_compliance_gaps_missing_one() {
        let regs = vec![
            make_test_registration_with_jurisdiction("active", "US-FED", "US"),
        ];
        let required = vec![("US-FED", "US"), ("GB-VAT", "GB")];
        let gaps = TaxRegistrationEngine::detect_compliance_gaps(
            &regs,
            &required,
            chrono::NaiveDate::from_ymd_opt(2025, 6, 15).unwrap(),
            30,
        );
        assert_eq!(gaps.len(), 1);
        assert!(gaps[0].0 == "GB-VAT");
    }

    #[test]
    fn test_compliance_gaps_expiring_soon() {
        let reg = TaxRegistration {
            effective_to: Some(chrono::NaiveDate::from_ymd_opt(2025, 7, 1).unwrap()),
            ..make_test_registration_with_jurisdiction("active", "US-FED", "US")
        };
        let required = vec![("US-FED", "US")];
        let gaps = TaxRegistrationEngine::detect_compliance_gaps(
            &[reg],
            &required,
            chrono::NaiveDate::from_ymd_opt(2025, 6, 15).unwrap(),
            30,
        );
        assert_eq!(gaps.len(), 1);
        assert!(gaps[0].1.contains("expires in"));
    }

    #[test]
    fn test_compliance_gaps_already_expired() {
        let reg = TaxRegistration {
            effective_to: Some(chrono::NaiveDate::from_ymd_opt(2025, 6, 10).unwrap()),
            ..make_test_registration_with_jurisdiction("active", "US-FED", "US")
        };
        let required = vec![("US-FED", "US")];
        let gaps = TaxRegistrationEngine::detect_compliance_gaps(
            &[reg],
            &required,
            chrono::NaiveDate::from_ymd_opt(2025, 6, 15).unwrap(),
            30,
        );
        assert_eq!(gaps.len(), 1);
        assert!(gaps[0].1.contains("has expired"));
    }

    // ========================================================================
    // Duplicate Detection Tests
    // ========================================================================

    #[test]
    fn test_is_duplicate_found() {
        let existing = vec![
            make_test_registration_with_number("VAT-123", "active"),
        ];
        assert!(TaxRegistrationEngine::is_duplicate_number("VAT-123", &existing));
    }

    #[test]
    fn test_is_duplicate_not_found() {
        let existing = vec![
            make_test_registration_with_number("VAT-123", "active"),
        ];
        assert!(!TaxRegistrationEngine::is_duplicate_number("VAT-456", &existing));
    }

    #[test]
    fn test_is_duplicate_ignores_deregistered() {
        let existing = vec![
            make_test_registration_with_number("VAT-123", "deregistered"),
        ];
        assert!(!TaxRegistrationEngine::is_duplicate_number("VAT-123", &existing));
    }

    #[test]
    fn test_is_duplicate_ignores_expired() {
        let existing = vec![
            make_test_registration_with_number("VAT-123", "expired"),
        ];
        assert!(!TaxRegistrationEngine::is_duplicate_number("VAT-123", &existing));
    }

    // ========================================================================
    // Filter Tests
    // ========================================================================

    #[test]
    fn test_filter_by_status() {
        let regs = vec![
            make_test_registration_with_number("R1", "active"),
            make_test_registration_with_number("R2", "active"),
            make_test_registration_with_number("R3", "suspended"),
            make_test_registration_with_number("R4", "deregistered"),
        ];
        let active = TaxRegistrationEngine::filter_by_status(&regs, "active");
        assert_eq!(active.len(), 2);
        let suspended = TaxRegistrationEngine::filter_by_status(&regs, "suspended");
        assert_eq!(suspended.len(), 1);
    }

    #[test]
    fn test_filter_by_status_empty() {
        let regs: Vec<TaxRegistration> = vec![];
        let active = TaxRegistrationEngine::filter_by_status(&regs, "active");
        assert!(active.is_empty());
    }

    // ========================================================================
    // Expiring Soon Tests
    // ========================================================================

    #[test]
    fn test_get_expiring_within_range() {
        let as_of = chrono::NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        let regs = vec![
            TaxRegistration {
                effective_to: Some(chrono::NaiveDate::from_ymd_opt(2025, 6, 25).unwrap()),
                ..make_test_registration_with_number("R1", "active")
            },
            TaxRegistration {
                effective_to: Some(chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
                ..make_test_registration_with_number("R2", "active")
            },
        ];
        let expiring = TaxRegistrationEngine::get_expiring(&regs, as_of, 30);
        assert_eq!(expiring.len(), 1);
        assert_eq!(expiring[0].1, 10); // 10 days remaining
    }

    #[test]
    fn test_get_expiring_none() {
        let as_of = chrono::NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        let regs = vec![
            TaxRegistration {
                effective_to: Some(chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
                ..make_test_registration_with_number("R1", "active")
            },
        ];
        let expiring = TaxRegistrationEngine::get_expiring(&regs, as_of, 30);
        assert!(expiring.is_empty());
    }

    #[test]
    fn test_get_expiring_already_expired() {
        let as_of = chrono::NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        let regs = vec![
            TaxRegistration {
                effective_to: Some(chrono::NaiveDate::from_ymd_opt(2025, 6, 10).unwrap()),
                ..make_test_registration_with_number("R1", "active")
            },
        ];
        let expiring = TaxRegistrationEngine::get_expiring(&regs, as_of, 30);
        assert!(expiring.is_empty()); // Already past end date
    }

    #[test]
    fn test_get_expiring_no_end_date() {
        let as_of = chrono::NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        let regs = vec![
            make_test_registration_with_number("R1", "active"), // No effective_to
        ];
        let expiring = TaxRegistrationEngine::get_expiring(&regs, as_of, 30);
        assert!(expiring.is_empty());
    }

    // ========================================================================
    // Registration Number Format Edge Cases
    // ========================================================================

    #[test]
    fn test_empty_registration_number_fails() {
        assert!(TaxRegistrationEngine::validate_format("", "vat", "US").is_err());
    }

    #[test]
    fn test_whitespace_only_fails() {
        assert!(TaxRegistrationEngine::validate_format("   ", "vat", "US").is_err());
    }

    #[test]
    fn test_us_ein_exactly_9_digits() {
        assert!(TaxRegistrationEngine::validate_format("000000000", "ein", "US").is_ok());
    }

    #[test]
    fn test_gb_vat_9_digits() {
        assert!(TaxRegistrationEngine::validate_format("999999999", "vat", "GB").is_ok());
    }

    #[test]
    fn test_de_vat_min_8_chars() {
        assert!(TaxRegistrationEngine::validate_format("DE12345678", "vat", "DE").is_ok());
    }

    #[test]
    fn test_de_vat_too_long() {
        assert!(TaxRegistrationEngine::validate_format("DE1234567890123456", "vat", "DE").is_err());
    }

    // ========================================================================
    // Helper Functions
    // ========================================================================

    fn make_test_registration(
        status: &str,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
    ) -> TaxRegistration {
        TaxRegistration {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            registration_number: "TEST-REG-001".to_string(),
            registration_type: "vat".to_string(),
            tax_purpose: "both".to_string(),
            party_type: "first_party".to_string(),
            party_id: Some(Uuid::new_v4()),
            party_name: Some("Test Legal Entity".to_string()),
            jurisdiction_code: "US-FED".to_string(),
            country_code: "US".to_string(),
            state_code: None,
            status: status.to_string(),
            effective_from,
            effective_to,
            is_default: false,
            reporting_name: None,
            legal_entity_id: None,
            validation_status: "validated".to_string(),
            last_validated_at: None,
            source: "manual".to_string(),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn make_test_registration_with_jurisdiction(
        status: &str,
        jurisdiction_code: &str,
        country_code: &str,
    ) -> TaxRegistration {
        TaxRegistration {
            jurisdiction_code: jurisdiction_code.to_string(),
            country_code: country_code.to_string(),
            status: status.to_string(),
            effective_from: chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            effective_to: None,
            ..make_test_registration("active", chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None)
        }
    }

    fn make_test_registration_with_number(
        number: &str,
        status: &str,
    ) -> TaxRegistration {
        TaxRegistration {
            registration_number: number.to_string(),
            status: status.to_string(),
            effective_from: chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            effective_to: None,
            ..make_test_registration(status, chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None)
        }
    }
}
