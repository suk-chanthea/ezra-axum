//! Church handlers, mirroring Go `ChurchHandler`.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::error::{AppError, AppResult};
use crate::interface::http::dto::request::{
    ApproveChurchMemberRequest, CreateChurchRequest, JoinChurchRequest, PaginationQuery,
};
use crate::interface::http::dto::response::SuccessResponse;
use crate::interface::http::dto::ValidatedJson;
use crate::interface::http::middleware::AuthUser;
use crate::state::AppState;

pub async fn create(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    ValidatedJson(req): ValidatedJson<CreateChurchRequest>,
) -> AppResult<impl IntoResponse> {
    let church = state.church.create_church(req, user_id).await?;
    let data = serde_json::to_value(church).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok((StatusCode::CREATED, Json(SuccessResponse::with_data("church created successfully", data))))
}

pub async fn get_all(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<impl IntoResponse> {
    let resp = state.church.get_all_churches(pagination.page, pagination.page_size).await?;
    Ok(Json(resp))
}

pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    let church = state.church.get_church_by_id(id).await?;
    Ok(Json(church))
}

pub async fn get_by_denomination(
    State(state): State<AppState>,
    Path(denomination): Path<String>,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<impl IntoResponse> {
    if denomination.is_empty() {
        return Err(AppError::BadRequest("denomination parameter is required".to_string()));
    }
    let resp = state
        .church
        .get_churches_by_denomination(&denomination, pagination.page, pagination.page_size)
        .await?;
    Ok(Json(resp))
}

pub async fn update(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
    ValidatedJson(req): ValidatedJson<CreateChurchRequest>,
) -> AppResult<impl IntoResponse> {
    let church = state.church.update_church(id, req, user_id).await?;
    let data = serde_json::to_value(church).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(SuccessResponse::with_data("church updated successfully", data)))
}

pub async fn delete(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    state.church.delete_church(id, user_id).await?;
    Ok(Json(SuccessResponse::message("church deleted successfully")))
}

pub async fn join_church(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    ValidatedJson(req): ValidatedJson<JoinChurchRequest>,
) -> AppResult<impl IntoResponse> {
    state.church.join_church(user_id, req.church_id).await?;
    Ok(Json(SuccessResponse::message(
        "church join request submitted. Waiting for owner approval",
    )))
}

pub async fn leave_church(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> AppResult<impl IntoResponse> {
    state.church.leave_church(user_id).await?;
    Ok(Json(SuccessResponse::message("left church successfully")))
}

pub async fn get_pending_members(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<impl IntoResponse> {
    let resp = state
        .church
        .get_pending_members(id, user_id, pagination.page, pagination.page_size)
        .await?;
    Ok(Json(resp))
}

pub async fn get_members(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<impl IntoResponse> {
    let resp = state.church.get_approved_members(id, pagination.page, pagination.page_size).await?;
    Ok(Json(resp))
}

pub async fn approve_member(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
    ValidatedJson(req): ValidatedJson<ApproveChurchMemberRequest>,
) -> AppResult<impl IntoResponse> {
    state.church.approve_member(id, user_id, req.user_id, &req.status).await?;
    let message = if req.status == "rejected" {
        "member rejected successfully"
    } else {
        "member approved successfully"
    };
    Ok(Json(SuccessResponse::message(message)))
}
