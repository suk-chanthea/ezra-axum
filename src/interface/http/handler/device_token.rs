//! Device token handlers (FCM registration), mirroring Go `DeviceTokenHandler`.

use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use validator::Validate;

use crate::domain::entity::DeviceToken;
use crate::error::{AppError, AppResult};
use crate::interface::http::dto::ValidatedJson;
use crate::interface::http::middleware::AuthUser;
use crate::state::AppState;

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterTokenRequest {
    #[validate(length(min = 1))]
    pub token: String,
    #[validate(length(min = 1))]
    pub platform: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UnregisterTokenRequest {
    #[validate(length(min = 1))]
    pub token: String,
}

pub async fn register_token(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    ValidatedJson(req): ValidatedJson<RegisterTokenRequest>,
) -> AppResult<impl IntoResponse> {
    let mut device_token = DeviceToken::new(user_id, req.token, req.platform);
    if !device_token.is_valid() {
        return Err(AppError::BadRequest("Invalid device token data".to_string()));
    }
    state
        .device_token_repo
        .save(&mut device_token)
        .await
        .map_err(|_| AppError::Internal("Failed to register device token".to_string()))?;

    Ok(Json(json!({
        "message": "Device token registered successfully",
        "token_id": device_token.id,
    })))
}

pub async fn unregister_token(
    State(state): State<AppState>,
    AuthUser(_user_id): AuthUser,
    ValidatedJson(req): ValidatedJson<UnregisterTokenRequest>,
) -> AppResult<impl IntoResponse> {
    state
        .device_token_repo
        .delete_token(&req.token)
        .await
        .map_err(|_| AppError::Internal("Failed to unregister device token".to_string()))?;
    Ok(Json(json!({ "message": "Device token unregistered successfully" })))
}

pub async fn delete_all_tokens(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> AppResult<impl IntoResponse> {
    state
        .device_token_repo
        .delete_user_tokens(user_id)
        .await
        .map_err(|_| AppError::Internal("Failed to delete device tokens".to_string()))?;
    Ok(Json(json!({ "message": "All device tokens deleted successfully" })))
}
