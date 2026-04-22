//! Middleware
//! 
//! Request/response middleware for authentication, logging, etc.

use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::{RwLock, Arc};

/// Authentication middleware that validates JWT tokens
pub async fn auth_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());
    
    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => return Err(StatusCode::UNAUTHORIZED),
    };
    
    // Decode token using handlers module
    let claims = crate::handlers::verify_token(token)?;
    
    // Insert claims into request extensions
    request.extensions_mut().insert(claims);
    
    Ok(next.run(request).await)
}

/// Admin authorization middleware – requires the JWT bearer to hold an
/// `admin` or `system` role.  Must be layered **after** `auth_middleware`
/// so that `Claims` are already present in request extensions.
pub async fn admin_auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let claims = request
        .extensions()
        .get::<crate::handlers::auth::Claims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !claims.roles.iter().any(|r| r == "admin" || r == "system") {
        tracing::warn!(
            "Admin access denied for user {} with roles {:?}",
            claims.sub,
            claims.roles
        );
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}

/// Simple in-memory rate limiter for login attempts
/// In production, use Redis or similar distributed store
pub struct RateLimiter {
    /// Map of IP -> (attempts, window_start)
    attempts: RwLock<HashMap<String, (u32, Instant)>>,
    /// Max attempts per window
    max_attempts: u32,
    /// Window duration in seconds
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_attempts: u32, window_secs: u64) -> Self {
        Self {
            attempts: RwLock::new(HashMap::new()),
            max_attempts,
            window: Duration::from_secs(window_secs),
        }
    }
    
    /// Check if a request from this IP should be allowed
    pub fn check(&self, ip: &str) -> bool {
        let now = Instant::now();
        
        // Clean old entries and check current state
        let mut attempts = self.attempts.write().unwrap();
        attempts.retain(|_, (_, start)| now.duration_since(*start) < self.window);
        
        // Check current state and determine action
        let entry = attempts.get(ip).cloned();
        
        match entry {
            Some((count, start)) if now.duration_since(start) < self.window => {
                if count >= self.max_attempts {
                    false
                } else {
                    attempts.insert(ip.to_string(), (count + 1, start));
                    true
                }
            }
            _ => {
                attempts.insert(ip.to_string(), (1, now));
                true
            }
        }
    }
    
    /// Get remaining attempts for an IP
    pub fn remaining(&self, ip: &str) -> u32 {
        let attempts = self.attempts.read().unwrap();
        match attempts.get(ip) {
            Some((count, start)) => {
                let now = Instant::now();
                if now.duration_since(*start) < self.window {
                    self.max_attempts.saturating_sub(*count)
                } else {
                    self.max_attempts
                }
            }
            None => self.max_attempts,
        }
    }
}

/// Extract client IP from request
pub fn get_client_ip(request: &Request) -> String {
    request
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Rate limiting middleware for login endpoint
pub async fn rate_limit_middleware(
    State(rate_limiter): State<Arc<RateLimiter>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let ip = get_client_ip(&request);
    
    if !rate_limiter.check(&ip) {
        tracing::warn!("Rate limit exceeded for IP: {}", ip);
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    
    Ok(next.run(request).await)
}

/// Global rate limiter instance
pub static LOGIN_RATE_LIMITER: std::sync::OnceLock<Arc<RateLimiter>> = 
    std::sync::OnceLock::new();

/// Get or create the login rate limiter
pub fn get_login_rate_limiter() -> Arc<RateLimiter> {
    LOGIN_RATE_LIMITER
        .get_or_init(|| Arc::new(RateLimiter::new(5, 300))) // 5 attempts per 5 minutes
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rate_limiter_allows_requests() {
        let limiter = RateLimiter::new(3, 60);
        
        assert!(limiter.check("192.168.1.1"));
        assert!(limiter.check("192.168.1.1"));
        assert!(limiter.check("192.168.1.1"));
        assert!(!limiter.check("192.168.1.1")); // 4th request should be blocked
    }
    
    #[test]
    fn test_rate_limiter_different_ips() {
        let limiter = RateLimiter::new(2, 60);
        
        assert!(limiter.check("192.168.1.1"));
        assert!(limiter.check("192.168.1.2")); // Different IP should be allowed
    }
}
