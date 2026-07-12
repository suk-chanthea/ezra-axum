//! Band handlers, mirroring Go `BandHandler`.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::error::AppResult;
use crate::interface::http::dto::request::{
    AddMusicsRequest, CreateBandRequest, PaginationQuery, ReorderMusicsRequest,
};
use crate::interface::http::dto::response::{PaginatedResponse, SuccessResponse};
use crate::interface::http::dto::ValidatedJson;
use crate::interface::http::middleware::AuthUser;
use crate::state::AppState;

pub async fn create(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(req): Json<CreateBandRequest>,
) -> AppResult<impl IntoResponse> {
    state.band.create_band(req, user_id).await?;
    Ok((StatusCode::CREATED, Json(SuccessResponse::message("band created successfully"))))
}

pub async fn get_all(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<impl IntoResponse> {
    if pagination.is_explicit() {
        let (data, meta) = state
            .band
            .get_all_bands_paginated(pagination.get_page(), pagination.get_page_size())
            .await?;
        return Ok(Json(PaginatedResponse { data, pagination: meta }).into_response());
    }
    let bands = state.band.get_all_bands().await?;
    Ok(Json(bands).into_response())
}

pub async fn get_by_user(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<impl IntoResponse> {
    if pagination.is_explicit() {
        let (data, meta) = state
            .band
            .get_bands_by_user_id_paginated(user_id, pagination.get_page(), pagination.get_page_size())
            .await?;
        return Ok(Json(PaginatedResponse { data, pagination: meta }).into_response());
    }
    let bands = state.band.get_bands_by_user_id(user_id).await?;
    Ok(Json(bands).into_response())
}

pub async fn get_public(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<impl IntoResponse> {
    if pagination.is_explicit() {
        let (data, meta) = state
            .band
            .get_public_bands_paginated(pagination.get_page(), pagination.get_page_size())
            .await?;
        return Ok(Json(PaginatedResponse { data, pagination: meta }).into_response());
    }
    let bands = state.band.get_public_bands().await?;
    Ok(Json(bands).into_response())
}

pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    let band = state.band.get_band_by_id(id).await?;
    Ok(Json(band))
}

pub async fn update(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
    Json(req): Json<CreateBandRequest>,
) -> AppResult<impl IntoResponse> {
    state.band.update_band(id, req, user_id).await?;
    Ok(Json(SuccessResponse::message("band updated successfully")))
}

pub async fn delete(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    state.band.delete_band(id, user_id).await?;
    Ok(Json(SuccessResponse::message("band deleted successfully")))
}

pub async fn add_musics(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
    ValidatedJson(req): ValidatedJson<AddMusicsRequest>,
) -> AppResult<impl IntoResponse> {
    state.band.add_musics_to_band(id, req.music_ids, user_id).await?;
    Ok((StatusCode::CREATED, Json(SuccessResponse::message("music added to band successfully"))))
}

pub async fn remove_music(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path((id, music_id)): Path<(i64, i64)>,
) -> AppResult<impl IntoResponse> {
    state.band.remove_music_from_band(id, music_id, user_id).await?;
    Ok(Json(SuccessResponse::message("music removed from band successfully")))
}

pub async fn get_musics(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    let musics = state.band.get_band_musics(id).await?;
    Ok(Json(musics))
}

pub async fn reorder_musics(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
    ValidatedJson(req): ValidatedJson<ReorderMusicsRequest>,
) -> AppResult<impl IntoResponse> {
    state.band.reorder_band_musics(id, req.music_orders, user_id).await?;
    Ok(Json(SuccessResponse::message("music order updated successfully")))
}

pub async fn get_members(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    let members = state.band.get_band_members(id).await?;
    Ok(Json(members))
}
