//! Authentication handlers

use axum::{
    extract::State,
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use chrono::{Utc, Duration};
use crate::AppState;
use std::sync::Arc;
use tracing::{info, debug, warn, error};
use uuid::Uuid;
use sqlx::FromRow;
use argon2::password_hash::{PasswordHash, PasswordVerifier, PasswordHasher};
use argon2::Argon2;

/// Error type used by handlers that return `(StatusCode, Json<Value>)` tuples.
type HandlerError = (StatusCode, Json<serde_json::Value>);

/// Parse a UUID from a claim string, returning a JSON error on failure.
///
/// Unlike `unwrap_or_default()`, this does NOT silently fall back to the nil
/// UUID — which would be an auth-scoping bypass.
pub fn parse_uuid(s: &str) -> Result<Uuid, HandlerError> {
    Uuid::parse_str(s).map_err(|_| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": "Invalid auth token"})))
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
    pub expires_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub roles: Vec<String>,
    pub org_id: String,
    pub exp: i64,
}

impl Claims {
    /// Parse the `org_id` claim into a `Uuid`, returning 500 on failure.
    ///
    /// Using this instead of `Uuid::parse_str(&claims.org_id).unwrap_or_default()`
    /// avoids silently falling back to the nil UUID on a malformed token.
    pub fn org_uuid(&self) -> Result<Uuid, StatusCode> {
        Uuid::parse_str(&self.org_id)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    /// Parse the `sub` claim into a `Uuid`, returning 500 on failure.
    pub fn user_uuid(&self) -> Result<Uuid, StatusCode> {
        Uuid::parse_str(&self.sub)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    /// Parse the `org_id` claim, returning a `(StatusCode, Json)` error.
    ///
    /// For handlers whose error type is `(StatusCode, Json<Value>)`.
    pub fn org_uuid_json(&self) -> Result<Uuid, HandlerError> {
        parse_uuid(&self.org_id)
    }

    /// Parse the `sub` claim, returning a `(StatusCode, Json)` error.
    pub fn user_uuid_json(&self) -> Result<Uuid, HandlerError> {
        parse_uuid(&self.sub)
    }
}

/// Token expiry duration in hours
const TOKEN_EXPIRY_HOURS: i64 = 8;

/// Dummy Argon2 hash used for timing-attack prevention when the user doesn't exist.
/// This is a valid hash that will never match any real password.
/// IMPORTANT: This hash was generated with a random salt and is NOT the same as
/// any seeded user password hash.
const DUMMY_ARGON2_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$TKQ1gSmV58RTlZwIXn1prg$qn2FnCqFmN6gmvzTrCh7fmq8DzxtJmJjI/FKD7vDzAo";

/// Represents a user row from the database
#[derive(Debug, FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    name: String,
    password_hash: String,
    /// Stored as text so it works whether the column is JSONB or TEXT[]
    roles_text: String,
    organization_id: Uuid,
}

/// Validates email format
fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.') && email.len() <= 255
}

/// Sanitizes email input
fn sanitize_email(email: &str) -> Option<String> {
    let email = email.trim().to_lowercase();
    if is_valid_email(&email) {
        Some(email)
    } else {
        None
    }
}

/// Login endpoint
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Validate email format
    let email = sanitize_email(&payload.email).ok_or_else(|| {
        warn!("Invalid email format: {}", payload.email);
        StatusCode::BAD_REQUEST
    })?;
    
    debug!("Login attempt for: {}", email);
    
    // Fetch user from database using proper column names
    let user_row = sqlx::query_as::<_, UserRow>(
        r#"
        SELECT id, email, name, password_hash,
               roles::text AS roles_text,
               organization_id 
        FROM _atlas.users 
        WHERE email = $1 AND is_active = true
        "#
    )
    .bind(&email)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Database error during login: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let user = match user_row {
        Some(row) => row,
        None => {
            // Perform dummy verification to prevent timing attacks
            // Use a valid Argon2 hash format (will never match real passwords)
            let _ = verify_password_internal(
                &payload.password,
                DUMMY_ARGON2_HASH
            );
            warn!("Login failed for unknown email: {}", email);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };
    
    // Verify password using Argon2
    verify_password_internal(&payload.password, &user.password_hash)
        .map_err(|_| {
            warn!("Invalid password for user: {}", email);
            StatusCode::UNAUTHORIZED
        })?;
    
    info!("User {} logged in successfully", email);
    
    let expires_at = Utc::now() + Duration::hours(TOKEN_EXPIRY_HOURS);
    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        roles: parse_roles(&user.roles_text),
        org_id: user.organization_id.to_string(),
        exp: expires_at.timestamp(),
    };
    
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    ).map_err(|e| {
        error!("JWT encoding error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    Ok(Json(LoginResponse {
        token,
        user: UserInfo {
            id: user.id.to_string(),
            email: user.email,
            name: user.name,
            roles: parse_roles(&user.roles_text),
        },
        expires_at: expires_at.to_rfc3339(),
    }))
}

/// Parse roles from the `roles` column.
/// Handles both JSONB (`["admin","system"]`) and TEXT[] (`{admin,system}`)
/// since the actual column type may differ depending on migration history.
fn parse_roles(raw: &str) -> Vec<String> {
    // Try JSON array first
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(raw) {
        return match val {
            serde_json::Value::Array(arr) => arr
                .into_iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
            serde_json::Value::String(s) => vec![s],
            _ => vec![],
        };
    }
    // PostgreSQL array literal: {admin,system}
    if raw.starts_with('{') && raw.ends_with('}') {
        return raw[1..raw.len() - 1]
            .split(',')
            .map(|s| s.trim().trim_matches('"').to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }
    vec![]
}

/// Internal password verification function
fn verify_password_internal(password: &str, hash: &str) -> Result<(), &'static str> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|_| "Invalid hash format")?;
    
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| "Invalid password")
}

/// Verify a JWT token and return the claims
///
/// Reads the JWT secret from the AppState global singleton so the same
/// secret used during `login()` is always used, avoiding the previous
/// inconsistency where the env var could change between calls.
pub fn verify_token(token: &str) -> Result<Claims, StatusCode> {
    // Use the canonical secret from AppState (set during startup)
    let jwt_secret = crate::state::APP_STATE.get()
        .map(|s| s.jwt_secret.clone())
        .ok_or_else(|| {
            tracing::error!("APP_STATE not initialized – cannot verify tokens");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    ).map_err(|e| {
        debug!("Token verification failed: {}", e);
        StatusCode::UNAUTHORIZED
    })?;

    Ok(token_data.claims)
}

/// Hash a password using Argon2 (for password storage)
/// Returns a PHC-formatted hash string
#[allow(dead_code)]
pub fn hash_password(password: &str) -> Result<String, StatusCode> {
    use argon2::password_hash::SaltString;
    use argon2::Argon2;
    use rand::RngCore;
    
    // Generate a random salt
    let mut salt_bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt_bytes);
    
    // Create salt string in PHC format
    let salt = SaltString::encode_b64(&salt_bytes)
        .map_err(|e| {
            error!("Salt generation error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Hash the password
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), &salt)
        .map_err(|e| {
            error!("Password hashing error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    Ok(hash.to_string())
}
