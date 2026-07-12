//! Booking repository (sqlx/PostgreSQL), preloading event (+musics) and user.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::Booking;
use crate::domain::repository::BookingRepository;
use crate::error::{AppError, AppResult};
use crate::infrastructure::persistence::rows::{BookingRow, EventRow, MusicRow, UserRow};

pub struct PgBookingRepository {
    pool: PgPool,
}

impl PgBookingRepository {
    pub fn new(pool: PgPool) -> Self {
        PgBookingRepository { pool }
    }

    async fn attach_relations(&self, booking: &mut Booking) -> AppResult<()> {
        if let Some(event_row) = sqlx::query_as::<_, EventRow>("SELECT * FROM events WHERE id = $1")
            .bind(booking.event_id)
            .fetch_optional(&self.pool)
            .await?
        {
            let mut event = event_row.into_entity();
            let musics = sqlx::query_as::<_, MusicRow>(
                r#"
                SELECT m.* FROM musics m
                JOIN event_musics em ON m.id = em.music_id
                WHERE em.event_id = $1
                ORDER BY em.display_order
                "#,
            )
            .bind(event.id)
            .fetch_all(&self.pool)
            .await?;
            event.music_ids = musics.iter().map(|m| m.id).collect();
            event.musics = musics.into_iter().map(MusicRow::into_entity).collect();
            booking.event = Some(event);
        }

        if let Some(user_row) = sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE id = $1")
            .bind(booking.user_id)
            .fetch_optional(&self.pool)
            .await?
        {
            booking.user = Some(user_row.into_entity());
        }

        Ok(())
    }

    async fn attach_many(&self, bookings: &mut [Booking]) -> AppResult<()> {
        for booking in bookings.iter_mut() {
            self.attach_relations(booking).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl BookingRepository for PgBookingRepository {
    async fn save(&self, booking: &mut Booking) -> AppResult<()> {
        let rec: (i64, DateTime<Utc>, DateTime<Utc>) = sqlx::query_as(
            r#"
            INSERT INTO bookings (event_id, user_id, status, notes, created_at, updated_at)
            VALUES ($1,$2,$3,$4, NOW(), NOW())
            RETURNING id, created_at, updated_at
            "#,
        )
        .bind(booking.event_id)
        .bind(booking.user_id)
        .bind(&booking.status)
        .bind(&booking.notes)
        .fetch_one(&self.pool)
        .await?;

        booking.id = rec.0;
        booking.created_at = rec.1;
        booking.updated_at = rec.2;
        Ok(())
    }

    async fn find_all(&self) -> AppResult<Vec<Booking>> {
        let rows = sqlx::query_as::<_, BookingRow>("SELECT * FROM bookings ORDER BY id")
            .fetch_all(&self.pool)
            .await?;
        let mut bookings: Vec<Booking> = rows.into_iter().map(BookingRow::into_entity).collect();
        self.attach_many(&mut bookings).await?;
        Ok(bookings)
    }

    async fn find_all_paginated(&self, offset: i64, limit: i64) -> AppResult<(Vec<Booking>, i64)> {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bookings")
            .fetch_one(&self.pool)
            .await?;
        let rows = sqlx::query_as::<_, BookingRow>("SELECT * FROM bookings ORDER BY id OFFSET $1 LIMIT $2")
            .bind(offset)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;
        let mut bookings: Vec<Booking> = rows.into_iter().map(BookingRow::into_entity).collect();
        self.attach_many(&mut bookings).await?;
        Ok((bookings, total))
    }

    async fn find_by_id(&self, id: i64) -> AppResult<Booking> {
        let row = sqlx::query_as::<_, BookingRow>("SELECT * FROM bookings WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("booking not found".to_string()))?;
        let mut booking = row.into_entity();
        self.attach_relations(&mut booking).await?;
        Ok(booking)
    }

    async fn find_by_event_id(&self, event_id: i64) -> AppResult<Vec<Booking>> {
        let rows = sqlx::query_as::<_, BookingRow>("SELECT * FROM bookings WHERE event_id = $1 ORDER BY id")
            .bind(event_id)
            .fetch_all(&self.pool)
            .await?;
        let mut bookings: Vec<Booking> = rows.into_iter().map(BookingRow::into_entity).collect();
        self.attach_many(&mut bookings).await?;
        Ok(bookings)
    }

    async fn find_by_user_id(&self, user_id: i64) -> AppResult<Vec<Booking>> {
        let rows = sqlx::query_as::<_, BookingRow>("SELECT * FROM bookings WHERE user_id = $1 ORDER BY id")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;
        let mut bookings: Vec<Booking> = rows.into_iter().map(BookingRow::into_entity).collect();
        self.attach_many(&mut bookings).await?;
        Ok(bookings)
    }

    async fn find_by_event_and_user(&self, event_id: i64, user_id: i64) -> AppResult<Booking> {
        let row = sqlx::query_as::<_, BookingRow>(
            "SELECT * FROM bookings WHERE event_id = $1 AND user_id = $2",
        )
        .bind(event_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("booking not found".to_string()))?;
        let mut booking = row.into_entity();
        self.attach_relations(&mut booking).await?;
        Ok(booking)
    }

    async fn update(&self, booking: &Booking) -> AppResult<()> {
        sqlx::query(
            "UPDATE bookings SET event_id = $1, user_id = $2, status = $3, notes = $4, updated_at = NOW() WHERE id = $5",
        )
        .bind(booking.event_id)
        .bind(booking.user_id)
        .bind(&booking.status)
        .bind(&booking.notes)
        .bind(booking.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: i64) -> AppResult<()> {
        sqlx::query("DELETE FROM bookings WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
