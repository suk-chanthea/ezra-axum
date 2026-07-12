//! Notification handlers, mirroring Go `NotificationHandler`.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use crate::error::AppResult;
use crate::interface::http::dto::request::{CreateNotificationRequest, PaginationQuery};
use crate::interface::http::dto::response::{PaginatedResponse, SuccessResponse};
use crate::interface::http::dto::ValidatedJson;
use crate::interface::http::middleware::AuthUser;
use crate::state::AppState;

pub async fn create(
    State(state): State<AppState>,
    AuthUser(sender_id): AuthUser,
    ValidatedJson(req): ValidatedJson<CreateNotificationRequest>,
) -> AppResult<impl IntoResponse> {
    let resp = state.notification.create_notification(sender_id, req).await?;
    Ok((StatusCode::CREATED, Json(resp)))
}

pub async fn create_band_notification(
    State(state): State<AppState>,
    AuthUser(sender_id): AuthUser,
    Path(band_id): Path<i64>,
    ValidatedJson(req): ValidatedJson<CreateNotificationRequest>,
) -> AppResult<impl IntoResponse> {
    let resp = state.notification.create_band_notification(sender_id, band_id, req).await?;
    Ok((StatusCode::CREATED, Json(resp)))
}

pub async fn create_broadcast(
    State(state): State<AppState>,
    AuthUser(sender_id): AuthUser,
    ValidatedJson(req): ValidatedJson<CreateNotificationRequest>,
) -> AppResult<impl IntoResponse> {
    let resp = state.notification.create_broadcast_notification(sender_id, req).await?;
    Ok((StatusCode::CREATED, Json(resp)))
}

pub async fn get_all(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<impl IntoResponse> {
    let (data, meta) = state
        .notification
        .get_notifications(user_id, pagination.get_page(), pagination.get_page_size())
        .await?;
    Ok(Json(PaginatedResponse { data, pagination: meta }))
}

pub async fn get_unread(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> AppResult<impl IntoResponse> {
    let resp = state.notification.get_unread_notifications(user_id).await?;
    Ok(Json(resp))
}

pub async fn get_unread_count(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> AppResult<impl IntoResponse> {
    let count = state.notification.get_unread_count(user_id).await?;
    Ok(Json(json!({ "count": count })))
}

pub async fn get_by_id(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    let resp = state.notification.get_notification_by_id(user_id, id).await?;
    Ok(Json(resp))
}

pub async fn mark_as_read(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    state.notification.mark_as_read(user_id, id).await?;
    Ok(Json(SuccessResponse::message("notification marked as read")))
}

pub async fn mark_all_as_read(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> AppResult<impl IntoResponse> {
    state.notification.mark_all_as_read(user_id).await?;
    Ok(Json(SuccessResponse::message("all notifications marked as read")))
}

pub async fn delete(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    state.notification.delete_notification(user_id, id).await?;
    Ok(Json(SuccessResponse::message("notification deleted successfully")))
}

pub async fn delete_all(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> AppResult<impl IntoResponse> {
    state.notification.delete_all_notifications(user_id).await?;
    Ok(Json(SuccessResponse::message("all notifications deleted successfully")))
}
