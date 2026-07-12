//! Music handlers, mirroring Go `MusicHandler`.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::error::AppResult;
use crate::interface::http::dto::request::{CreateMusicRequest, PaginationQuery};
use crate::interface::http::dto::response::{PaginatedResponse, SuccessResponse};
use crate::interface::http::middleware::AuthUser;
use crate::state::AppState;

pub async fn create(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<CreateMusicRequest>,
) -> AppResult<impl IntoResponse> {
    state.music.create_music(req.title, req.cover, user_id).await?;
    Ok((StatusCode::CREATED, Json(SuccessResponse::message("music created successfully"))))
}

pub async fn get_all(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<impl IntoResponse> {
    if pagination.is_explicit() {
        let (data, meta) = state
            .music
            .get_all_musics_paginated(pagination.get_page(), pagination.get_page_size())
            .await?;
        return Ok(Json(PaginatedResponse { data, pagination: meta }).into_response());
    }
    let musics = state.music.get_all_musics().await?;
    Ok(Json(musics).into_response())
}

pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    let music = state.music.get_music_by_id(id).await?;
    Ok(Json(music))
}

pub async fn get_by_user(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> AppResult<impl IntoResponse> {
    let musics = state.music.get_musics_by_user_id(user_id).await?;
    Ok(Json(musics))
}

pub async fn update(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
    Json(req): Json<CreateMusicRequest>,
) -> AppResult<impl IntoResponse> {
    state.music.update_music(id, req.title, req.cover, user_id).await?;
    Ok(Json(SuccessResponse::message("music updated successfully")))
}

pub async fn delete(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    state.music.delete_music(id, user_id).await?;
    Ok(Json(SuccessResponse::message("music deleted successfully")))
}
