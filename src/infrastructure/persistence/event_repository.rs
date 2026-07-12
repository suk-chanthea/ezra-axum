//! Event repository (sqlx/PostgreSQL), including the event_musics many-to-many join.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::{Event, Music};
use crate::domain::repository::EventRepository;
use crate::error::{AppError, AppResult};
use crate::infrastructure::persistence::rows::{EventRow, MusicRow};

pub struct PgEventRepository {
    pool: PgPool,
}

impl PgEventRepository {
    pub fn new(pool: PgPool) -> Self {
        PgEventRepository { pool }
    }

    async fn load_musics(&self, event_id: i64) -> AppResult<Vec<Music>> {
        let rows = sqlx::query_as::<_, MusicRow>(
            r#"
            SELECT m.* FROM musics m
            JOIN event_musics em ON m.id = em.music_id
            WHERE em.event_id = $1
            ORDER BY em.display_order
            "#,
        )
        .bind(event_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(MusicRow::into_entity).collect())
    }

    async fn attach_musics(&self, event: &mut Event) -> AppResult<()> {
        let musics = self.load_musics(event.id).await?;
        event.music_ids = musics.iter().map(|m| m.id).collect();
        event.musics = musics;
        Ok(())
    }
}

#[async_trait]
impl EventRepository for PgEventRepository {
    async fn save(&self, event: &mut Event) -> AppResult<()> {
        let mut tx = self.pool.begin().await?;

        let rec: (i64, DateTime<Utc>, DateTime<Utc>) = sqlx::query_as(
            r#"
            INSERT INTO events
                (title, content, cover, location, start_time, end_time, user_id, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7, NOW(), NOW())
            RETURNING id, created_at, updated_at
            "#,
        )
        .bind(&event.title)
        .bind(&event.content)
        .bind(&event.cover)
        .bind(&event.location)
        .bind(event.start_time)
        .bind(event.end_time)
        .bind(event.user_id)
        .fetch_one(&mut *tx)
        .await?;

        let event_id = rec.0;

        for (i, music_id) in event.music_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO event_musics (event_id, music_id, display_order, created_at) VALUES ($1,$2,$3, NOW())",
            )
            .bind(event_id)
            .bind(music_id)
            .bind(i as i32)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        event.id = event_id;
        event.created_at = rec.1;
        event.updated_at = rec.2;
        Ok(())
    }

    async fn find_all(&self) -> AppResult<Vec<Event>> {
        let rows = sqlx::query_as::<_, EventRow>("SELECT * FROM events ORDER BY id")
            .fetch_all(&self.pool)
            .await?;
        let mut events: Vec<Event> = rows.into_iter().map(EventRow::into_entity).collect();
        for event in events.iter_mut() {
            self.attach_musics(event).await?;
        }
        Ok(events)
    }

    async fn find_all_paginated(&self, offset: i64, limit: i64) -> AppResult<(Vec<Event>, i64)> {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events")
            .fetch_one(&self.pool)
            .await?;
        let rows = sqlx::query_as::<_, EventRow>("SELECT * FROM events ORDER BY id OFFSET $1 LIMIT $2")
            .bind(offset)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;
        let mut events: Vec<Event> = rows.into_iter().map(EventRow::into_entity).collect();
        for event in events.iter_mut() {
            self.attach_musics(event).await?;
        }
        Ok((events, total))
    }

    async fn find_by_id(&self, id: i64) -> AppResult<Event> {
        let row = sqlx::query_as::<_, EventRow>("SELECT * FROM events WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("event not found".to_string()))?;
        let mut event = row.into_entity();
        self.attach_musics(&mut event).await?;
        Ok(event)
    }

    async fn find_by_user_id(&self, user_id: i64) -> AppResult<Vec<Event>> {
        let rows = sqlx::query_as::<_, EventRow>("SELECT * FROM events WHERE user_id = $1 ORDER BY id")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;
        let mut events: Vec<Event> = rows.into_iter().map(EventRow::into_entity).collect();
        for event in events.iter_mut() {
            self.attach_musics(event).await?;
        }
        Ok(events)
    }

    async fn update(&self, event: &Event) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE events SET
                title = $1, content = $2, cover = $3, location = $4, start_time = $5,
                end_time = $6, user_id = $7, updated_at = NOW()
            WHERE id = $8
            "#,
        )
        .bind(&event.title)
        .bind(&event.content)
        .bind(&event.cover)
        .bind(&event.location)
        .bind(event.start_time)
        .bind(event.end_time)
        .bind(event.user_id)
        .bind(event.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: i64) -> AppResult<()> {
        sqlx::query("DELETE FROM events WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn add_musics_to_event(&self, event_id: i64, music_ids: &[i64]) -> AppResult<()> {
        for (i, music_id) in music_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO event_musics (event_id, music_id, display_order, created_at) VALUES ($1,$2,$3, NOW())",
            )
            .bind(event_id)
            .bind(music_id)
            .bind(i as i32)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    async fn remove_musics_from_event(&self, event_id: i64, music_ids: &[i64]) -> AppResult<()> {
        sqlx::query("DELETE FROM event_musics WHERE event_id = $1 AND music_id = ANY($2)")
            .bind(event_id)
            .bind(music_ids)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_event_musics(&self, event_id: i64) -> AppResult<Vec<Music>> {
        self.load_musics(event_id).await
    }
}
