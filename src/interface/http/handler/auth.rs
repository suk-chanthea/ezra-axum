//! Auth handlers (register, login, OAuth, profile), mirroring Go `AuthHandler`.

use axum::extract::{Query, State, Path};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;

use crate::error::{AppError, AppResult};
use crate::infrastructure::security::google::GoogleVerifier;
use crate::interface::http::dto::response::SuccessResponse;
use crate::interface::http::dto::request::{
    GoogleLoginRequest, LoginRequest, RegisterRequest, ResetPasswordRequest, UpdateProfileRequest,
};
use crate::interface::http::dto::ValidatedJson;
use crate::interface::http::middleware::AuthUser;
use crate::state::AppState;

pub async fn register(
    State(state): State<AppState>,
    ValidatedJson(req): ValidatedJson<RegisterRequest>,
) -> AppResult<impl IntoResponse> {
    let resp = state.auth.register(req).await?;
    Ok((StatusCode::CREATED, Json(resp)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<impl IntoResponse> {
    let resp = state.auth.login(req).await?;
    Ok(Json(resp))
}

pub async fn google_login(
    State(state): State<AppState>,
    Json(req): Json<GoogleLoginRequest>,
) -> AppResult<impl IntoResponse> {
    let verifier = GoogleVerifier::new();
    let claims = verifier.validate(&req.id_token).await?;

    if claims.sub.is_empty() {
        return Err(AppError::BadRequest("invalid token: missing subject".to_string()));
    }
    if claims.email.is_empty() {
        return Err(AppError::BadRequest("invalid token: missing email".to_string()));
    }
    let name = if claims.name.is_empty() { claims.email.clone() } else { claims.name.clone() };

    let resp = state
        .auth
        .google_login(&claims.sub, &claims.email, &name, &claims.picture)
        .await?;
    Ok(Json(resp))
}

pub async fn reset_password(
    State(state): State<AppState>,
    ValidatedJson(req): ValidatedJson<ResetPasswordRequest>,
) -> AppResult<impl IntoResponse> {
    let resp = state.auth.reset_password(req).await?;
    Ok(Json(resp))
}

#[derive(serde::Deserialize)]
pub struct LogoutQuery {
    #[serde(default)]
    pub all: bool,
}

pub async fn logout(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Query(query): Query<LogoutQuery>,
    headers: HeaderMap,
) -> AppResult<impl IntoResponse> {
    if query.all {
        state.auth.logout_all(user_id).await?;
        Ok(Json(SuccessResponse::message("logged out from all devices successfully")))
    } else {
        let token = extract_token_from_headers(&headers)?;
        state.auth.logout_session(user_id, &token).await?;
        Ok(Json(SuccessResponse::message("logged out successfully")))
    }
}

pub async fn logout_all(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> AppResult<impl IntoResponse> {
    state.auth.logout_all(user_id).await?;
    Ok(Json(SuccessResponse::message("logged out from all devices successfully")))
}

fn extract_token_from_headers(headers: &HeaderMap) -> AppResult<String> {
    let header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Authorization header required".to_string()))?;

    let parts_vec: Vec<&str> = header.split(' ').collect();
    if parts_vec.len() != 2 || parts_vec[0] != "Bearer" {
        return Err(AppError::Unauthorized("Invalid authorization format".to_string()));
    }
    Ok(parts_vec[1].to_string())
}

pub async fn delete_user(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> AppResult<impl IntoResponse> {
    state.auth.delete_user(user_id).await?;
    Ok(Json(SuccessResponse::message("user deleted successfully")))
}

pub async fn get_me(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> AppResult<impl IntoResponse> {
    let user = state.auth.get_me(user_id).await?;
    Ok(Json(user))
}

pub async fn update_me(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    ValidatedJson(req): ValidatedJson<UpdateProfileRequest>,
) -> AppResult<impl IntoResponse> {
    let user = state.auth.update_me(user_id, req).await?;
    let data = serde_json::to_value(user).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(SuccessResponse::with_data("profile updated successfully", data)))
}

pub async fn get_sessions(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> AppResult<impl IntoResponse> {
    let sessions = state.auth.get_active_sessions(user_id).await?;
    Ok(Json(sessions))
}

pub async fn revoke_session(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(session_id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    state.auth.revoke_session(user_id, session_id).await?;
    Ok(Json(SuccessResponse::message("session revoked successfully")))
}
