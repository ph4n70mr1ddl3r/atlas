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
use std::time::{SystemTime, UNIX_EPOCH};

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

/// Token expiry duration in hours
const TOKEN_EXPIRY_HOURS: i64 = 8;

/// Represents a user row from the database
#[derive(Debug, FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    name: String,
    password_hash: String,
    roles: sqlx::types::Json<Vec<String>>,
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
        SELECT id, email, name, password_hash, roles, organization_id 
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
            // A properly hashed password would be 60+ chars
            let _ = verify_password_internal(
                &payload.password, 
                "$argon2id$v=19$m=19456,t=2,p=1$ZGFtcGR1bW15c2FsdHRlY29ustantdGhpcw$YWCxLjRVaG1GUmI5ZmxhemUxaXBsb2JqZWZ0"
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
        roles: user.roles.0.clone(),
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
            roles: user.roles.0,
        },
        expires_at: expires_at.to_rfc3339(),
    }))
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
        .unwrap_or_else(|| {
            std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "dev-secret-key-please-change-in-production-1234567890".to_string())
        });

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    ).map_err(|e| {
        debug!("Token verification failed: {}", e);
        StatusCode::UNAUTHORIZED
    })?;

    // Check expiration (belt-and-suspenders; `Validation::default()` already checks exp)
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    if token_data.claims.exp < now {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(token_data.claims)
}

/// Hash a password using Argon2 (for password storage)
/// Returns a PHC-formatted hash string
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
