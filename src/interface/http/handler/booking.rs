//! Booking handlers, mirroring Go `BookingHandler`.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::error::AppResult;
use crate::interface::http::dto::request::{
    CreateBookingRequest, PaginationQuery, UpdateBookingRequest,
};
use crate::interface::http::dto::response::{PaginatedResponse, SuccessResponse};
use crate::interface::http::dto::ValidatedJson;
use crate::interface::http::middleware::AuthUser;
use crate::state::AppState;

pub async fn create(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    ValidatedJson(req): ValidatedJson<CreateBookingRequest>,
) -> AppResult<impl IntoResponse> {
    state.booking.create_booking(req.event_id, req.notes, user_id).await?;
    Ok((StatusCode::CREATED, Json(SuccessResponse::message("booking created successfully"))))
}

pub async fn get_all(
    State(state): State<AppState>,
    Query(pagination): Query<PaginationQuery>,
) -> AppResult<impl IntoResponse> {
    if pagination.is_explicit() {
        let (data, meta) = state
            .booking
            .get_all_bookings_paginated(pagination.get_page(), pagination.get_page_size())
            .await?;
        return Ok(Json(PaginatedResponse { data, pagination: meta }).into_response());
    }
    let bookings = state.booking.get_all_bookings().await?;
    Ok(Json(bookings).into_response())
}

pub async fn get_by_id(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    let booking = state.booking.get_booking_by_id(id).await?;
    Ok(Json(booking))
}

pub async fn get_by_event(
    State(state): State<AppState>,
    Path(event_id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    let bookings = state.booking.get_bookings_by_event_id(event_id).await?;
    Ok(Json(bookings))
}

pub async fn get_by_user(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> AppResult<impl IntoResponse> {
    let bookings = state.booking.get_bookings_by_user_id(user_id).await?;
    Ok(Json(bookings))
}

pub async fn update(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
    ValidatedJson(req): ValidatedJson<UpdateBookingRequest>,
) -> AppResult<impl IntoResponse> {
    state.booking.update_booking(id, req.status, req.notes, user_id).await?;
    Ok(Json(SuccessResponse::message("booking updated successfully")))
}

pub async fn delete(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<i64>,
) -> AppResult<impl IntoResponse> {
    state.booking.delete_booking(id, user_id).await?;
    Ok(Json(SuccessResponse::message("booking deleted successfully")))
}
