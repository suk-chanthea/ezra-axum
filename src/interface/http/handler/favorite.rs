//! Favorite handlers, mirroring Go `FavoriteHandler`.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use crate::error::{AppError, AppResult};
use crate::interface::http::dto::request::PaginationQuery;
use crate::interface::http::dto::response::{PaginatedResponse, SuccessResponse};
use crate::interface::http::middleware::AuthUser;
use crate::state::AppState;

pub async fn add_favorite(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    match state.favorite.add_favorite(user_id, id).await {
        Ok(()) => Ok((StatusCode::CREATED, Json(SuccessResponse::message("music added to favorites")))),
        Err(AppError::BadRequest(msg)) if msg == "music not found" => Err(AppError::NotFound(msg)),
        Err(AppError::BadRequest(msg)) if msg == "already favorited" => Err(AppError::Conflict(msg)),
        Err(e) => Err(e),
    }
}

pub async fn remove_favorite(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    match state.favorite.remove_favorite(user_id, id).await {
        Ok(()) => Ok(Json(SuccessResponse::message("music removed from favorites"))),
        Err(AppError::BadRequest(msg)) if msg == "favorite not found" => Err(AppError::NotFound(msg)),
        Err(e) => Err(e),
    }
}

pub async fn get_user_favorites(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<impl IntoResponse> {
    if pagination.is_explicit() {
        let (data, meta) = state
            .favorite
            .get_user_favorites_paginated(user_id, pagination.get_page(), pagination.get_page_size())
            .await?;
        return Ok(Json(PaginatedResponse { data, pagination: meta }).into_response());
    }
    let favorites = state.favorite.get_user_favorites(user_id).await?;
    Ok(Json(favorites).into_response())
}

pub async fn is_favorite(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    let is_fav = state.favorite.is_favorite(user_id, id).await?;
    Ok(Json(json!({ "is_favorite": is_fav })))
}

pub async fn get_favorite_count(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    let count = state.favorite.get_favorite_count(id).await?;
    Ok(Json(json!({ "count": count })))
}
