//! Booking use case, mirroring Go `bookingUseCase`.

use std::sync::Arc;

use crate::domain::entity::Booking;
use crate::domain::repository::{BookingRepository, EventRepository};
use crate::error::{AppError, AppResult};
use crate::interface::http::dto::response::{BookingResponse, PaginationMetadata};

pub struct BookingUseCase {
    booking_repo: Arc<dyn BookingRepository>,
    event_repo: Arc<dyn EventRepository>,
}

impl BookingUseCase {
    pub fn new(
        booking_repo: Arc<dyn BookingRepository>,
        event_repo: Arc<dyn EventRepository>,
    ) -> Self {
        BookingUseCase { booking_repo, event_repo }
    }

    pub async fn create_booking(&self, event_id: i64, notes: String, user_id: i64) -> AppResult<()> {
        let event = self
            .event_repo
            .find_by_id(event_id)
            .await
            .map_err(|_| AppError::BadRequest("event not found".to_string()))?;

        if event.user_id == user_id {
            return Err(AppError::BadRequest("cannot book your own event".to_string()));
        }

        if self.booking_repo.find_by_event_and_user(event_id, user_id).await.is_ok() {
            return Err(AppError::BadRequest("you have already booked this event".to_string()));
        }

        let mut booking = Booking::new(event_id, user_id, notes);
        if !booking.is_valid() {
            return Err(AppError::BadRequest("invalid booking data".to_string()));
        }
        self.booking_repo.save(&mut booking).await
    }

    pub async fn get_all_bookings(&self) -> AppResult<Vec<BookingResponse>> {
        let bookings = self.booking_repo.find_all().await?;
        Ok(BookingResponse::list(&bookings))
    }

    pub async fn get_all_bookings_paginated(
        &self,
        page: i64,
        page_size: i64,
    ) -> AppResult<(Vec<BookingResponse>, PaginationMetadata)> {
        let offset = (page - 1) * page_size;
        let (bookings, total) = self.booking_repo.find_all_paginated(offset, page_size).await?;
        Ok((BookingResponse::list(&bookings), PaginationMetadata::new(page, page_size, total)))
    }

    pub async fn get_booking_by_id(&self, id: i64) -> AppResult<BookingResponse> {
        let booking = self.booking_repo.find_by_id(id).await?;
        Ok(BookingResponse::from_entity(&booking))
    }

    pub async fn get_bookings_by_event_id(&self, event_id: i64) -> AppResult<Vec<BookingResponse>> {
        let bookings = self.booking_repo.find_by_event_id(event_id).await?;
        Ok(BookingResponse::list(&bookings))
    }

    pub async fn get_bookings_by_user_id(&self, user_id: i64) -> AppResult<Vec<BookingResponse>> {
        let bookings = self.booking_repo.find_by_user_id(user_id).await?;
        Ok(BookingResponse::list(&bookings))
    }

    pub async fn update_booking(
        &self,
        id: i64,
        status: String,
        notes: String,
        user_id: i64,
    ) -> AppResult<()> {
        let mut booking = self
            .booking_repo
            .find_by_id(id)
            .await
            .map_err(|_| AppError::BadRequest("booking not found".to_string()))?;
        if booking.user_id != user_id {
            return Err(AppError::BadRequest("unauthorized to update this booking".to_string()));
        }
        booking.status = status;
        booking.notes = notes;
        self.booking_repo.update(&booking).await
    }

    pub async fn delete_booking(&self, id: i64, user_id: i64) -> AppResult<()> {
        let booking = self
            .booking_repo
            .find_by_id(id)
            .await
            .map_err(|_| AppError::BadRequest("booking not found".to_string()))?;
        if booking.user_id != user_id {
            return Err(AppError::BadRequest("unauthorized to delete this booking".to_string()));
        }
        self.booking_repo.delete(id).await
    }
}
