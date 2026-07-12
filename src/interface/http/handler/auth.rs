//! Auth handlers (register, login, OAuth, profile), mirroring Go `AuthHandler`.

use axum::extract::State;
use axum::http::StatusCode;
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
    let fullname = if claims.name.is_empty() { claims.email.clone() } else { claims.name.clone() };

    let resp = state
        .auth
        .google_login(&claims.sub, &claims.email, &fullname, &claims.picture)
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

pub async fn logout(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> AppResult<impl IntoResponse> {
    state.auth.logout(user_id).await?;
    Ok(Json(SuccessResponse::message("logged out successfully")))
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
