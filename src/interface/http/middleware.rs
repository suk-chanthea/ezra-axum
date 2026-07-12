//! JWT authentication, ported from the Go `JWTMiddleware`. Implemented as an Axum
//! extractor so protected handlers simply take an `AuthUser` argument.

use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;

use crate::error::AppError;
use crate::state::AppState;

/// Authenticated user id, resolved from a validated Bearer token.
pub struct AuthUser(pub i64);

#[axum::async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Authorization header required".to_string()))?;

        let parts_vec: Vec<&str> = header.split(' ').collect();
        if parts_vec.len() != 2 || parts_vec[0] != "Bearer" {
            return Err(AppError::Unauthorized("Invalid authorization format".to_string()));
        }
        let token = parts_vec[1];

        let claims = state
            .auth
            .validate_token(token)
            .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

        state.auth.verify_token_in_database(claims.sub, token).await?;

        Ok(AuthUser(claims.sub))
    }
}
