//! JWT issuing/validation (HS256), mirroring the Go `golang-jwt` claims (sub, username, role, exp).

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub username: String,
    pub role: String,
    pub exp: i64,
}

#[derive(Clone)]
pub struct JwtManager {
    secret: Vec<u8>,
}

impl JwtManager {
    pub fn new(secret: &str) -> Self {
        JwtManager {
            secret: secret.as_bytes().to_vec(),
        }
    }

    /// Issues a token valid for ~3 months, matching the Go `AddDate(0, 3, 0)` expiry.
    pub fn generate(&self, user_id: i64, username: &str, role: &str) -> AppResult<String> {
        let exp = (Utc::now() + Duration::days(90)).timestamp();
        let claims = Claims {
            sub: user_id,
            username: username.to_string(),
            role: role.to_string(),
            exp,
        };
        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(&self.secret),
        )
        .map_err(|e| AppError::Internal(e.to_string()))
    }

    /// Validates the signature and expiry, returning the decoded claims.
    pub fn validate(&self, token: &str) -> AppResult<Claims> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        decode::<Claims>(token, &DecodingKey::from_secret(&self.secret), &validation)
            .map(|data| data.claims)
            .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))
    }
}
