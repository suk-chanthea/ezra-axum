//! Event handlers, mirroring Go `EventHandler`.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::error::AppResult;
use crate::interface::http::dto::request::{CreateEventRequest, PaginationQuery};
use crate::interface::http::dto::response::{PaginatedResponse, SuccessResponse};
use crate::interface::http::dto::ValidatedJson;
use crate::interface::http::middleware::AuthUser;
use crate::state::AppState;

pub async fn create(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    ValidatedJson(req): ValidatedJson<CreateEventRequest>,
) -> AppResult<impl IntoResponse> {
    state.event.create_event(req, user_id).await?;
    Ok((StatusCode::CREATED, Json(SuccessResponse::message("event created successfully"))))
}

pub async fn get_all(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<impl IntoResponse> {
    if pagination.is_explicit() {
        let (data, meta) = state
            .event
            .get_all_events_paginated(pagination.get_page(), pagination.get_page_size())
            .await?;
        return Ok(Json(PaginatedResponse { data, pagination: meta }).into_response());
    }
    let events = state.event.get_all_events().await?;
    Ok(Json(events).into_response())
}

pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    let event = state.event.get_event_by_id(id).await?;
    Ok(Json(event))
}

pub async fn get_by_user(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> AppResult<impl IntoResponse> {
    let events = state.event.get_events_by_user_id(user_id).await?;
    Ok(Json(events))
}

pub async fn update(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
    ValidatedJson(req): ValidatedJson<CreateEventRequest>,
) -> AppResult<impl IntoResponse> {
    state.event.update_event(id, req, user_id).await?;
    Ok(Json(SuccessResponse::message("event updated successfully")))
}

pub async fn delete(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    state.event.delete_event(id, user_id).await?;
    Ok(Json(SuccessResponse::message("event deleted successfully")))
}
