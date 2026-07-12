//! Supporter handlers, mirroring Go `SupporterHandler`.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::error::{AppError, AppResult};
use crate::interface::http::dto::request::{
    CreateSupporterRequest, EmailQuery, PaginationQuery, UpdateSupporterRequest,
};
use crate::interface::http::dto::response::SuccessResponse;
use crate::interface::http::dto::ValidatedJson;
use crate::interface::http::middleware::AuthUser;
use crate::state::AppState;

pub async fn create(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    ValidatedJson(req): ValidatedJson<CreateSupporterRequest>,
) -> AppResult<impl IntoResponse> {
    let supporter = state.supporter.create_supporter(req, Some(user_id)).await?;
    let data = serde_json::to_value(supporter).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok((StatusCode::CREATED, Json(SuccessResponse::with_data("supporter created successfully", data))))
}

pub async fn get_all(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<impl IntoResponse> {
    let resp = state.supporter.get_all_supporters(pagination.page, pagination.page_size).await?;
    Ok(Json(resp))
}

pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    let supporter = state.supporter.get_supporter_by_id(id).await?;
    Ok(Json(supporter))
}

pub async fn get_by_email(
    State(state): State<AppState>,
    Query(query): Query<EmailQuery>,
) -> AppResult<impl IntoResponse> {
    if query.email.is_empty() {
        return Err(AppError::BadRequest("email parameter is required".to_string()));
    }
    let supporter = state.supporter.get_supporter_by_email(&query.email).await?;
    Ok(Json(supporter))
}

pub async fn get_by_type(
    State(state): State<AppState>,
    Path(supporter_type): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<impl IntoResponse> {
    if supporter_type != "company" && supporter_type != "organization" && supporter_type != "church" {
        return Err(AppError::BadRequest("invalid supporter type".to_string()));
    }
    let resp = state
        .supporter
        .get_supporters_by_type(&supporter_type, pagination.page, pagination.page_size)
        .await?;
    Ok(Json(resp))
}

pub async fn get_by_user(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<impl IntoResponse> {
    let supporters = state
        .supporter
        .get_supporters_by_user(user_id, pagination.page, pagination.page_size)
        .await?;
    Ok(Json(supporters))
}

pub async fn update(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
    ValidatedJson(req): ValidatedJson<UpdateSupporterRequest>,
) -> AppResult<impl IntoResponse> {
    let supporter = state.supporter.update_supporter(id, req, Some(user_id)).await?;
    let data = serde_json::to_value(supporter).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(SuccessResponse::with_data("supporter updated successfully", data)))
}

pub async fn delete(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    state.supporter.delete_supporter(id, Some(user_id)).await?;
    Ok(Json(SuccessResponse::message("supporter deleted successfully")))
}

pub async fn get_stats(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    let stats = state.supporter.get_supporter_stats(id).await?;
    Ok(Json(stats))
}
