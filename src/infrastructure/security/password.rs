//! Bcrypt password hashing, mirroring Go's `golang.org/x/crypto/bcrypt` usage.

use crate::error::{AppError, AppResult};

pub fn hash_password(plain: &str) -> AppResult<String> {
    bcrypt::hash(plain, bcrypt::DEFAULT_COST).map_err(|e| AppError::Internal(e.to_string()))
}

/// Returns true if the password matches the bcrypt hash.
pub fn verify_password(plain: &str, hash: &str) -> bool {
    bcrypt::verify(plain, hash).unwrap_or(false)
}
