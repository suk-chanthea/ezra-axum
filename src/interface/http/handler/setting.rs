//! Setting handlers, mirroring Go `SettingHandler`.

use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;

use crate::error::AppResult;
use crate::interface::http::dto::request::UpdateSettingRequest;
use crate::interface::http::dto::ValidatedJson;
use crate::interface::http::middleware::AuthUser;
use crate::state::AppState;

pub async fn get_settings(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> AppResult<impl IntoResponse> {
    let resp = state.setting.get_user_settings(user_id).await?;
    Ok(Json(resp))
}

pub async fn update_settings(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    ValidatedJson(req): ValidatedJson<UpdateSettingRequest>,
) -> AppResult<impl IntoResponse> {
    let resp = state.setting.update_settings(user_id, req).await?;
    Ok(Json(resp))
}

pub async fn reset_settings(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> AppResult<impl IntoResponse> {
    let resp = state.setting.reset_to_defaults(user_id).await?;
    Ok(Json(resp))
}
