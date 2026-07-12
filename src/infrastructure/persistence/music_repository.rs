//! Music repository (sqlx/PostgreSQL).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::Music;
use crate::domain::repository::MusicRepository;
use crate::error::{AppError, AppResult};
use crate::infrastructure::persistence::rows::MusicRow;

pub struct PgMusicRepository {
    pool: PgPool,
}

impl PgMusicRepository {
    pub fn new(pool: PgPool) -> Self {
        PgMusicRepository { pool }
    }
}

#[async_trait]
impl MusicRepository for PgMusicRepository {
    async fn save(&self, music: &mut Music) -> AppResult<()> {
        let rec: (i64, DateTime<Utc>, DateTime<Utc>) = sqlx::query_as(
            r#"
            INSERT INTO musics
                (title, artist, album, genre, duration, bpm, key, cover, lyrics, description,
                 user_id, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11, NOW(), NOW())
            RETURNING id, created_at, updated_at
            "#,
        )
        .bind(&music.title)
        .bind(&music.artist)
        .bind(&music.album)
        .bind(&music.genre)
        .bind(music.duration)
        .bind(music.bpm)
        .bind(&music.key)
        .bind(&music.cover)
        .bind(&music.lyrics)
        .bind(&music.description)
        .bind(music.user_id)
        .fetch_one(&self.pool)
        .await?;

        music.id = rec.0;
        music.created_at = rec.1;
        music.updated_at = rec.2;
        Ok(())
    }

    async fn find_all(&self) -> AppResult<Vec<Music>> {
        let rows = sqlx::query_as::<_, MusicRow>("SELECT * FROM musics ORDER BY id")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(MusicRow::into_entity).collect())
    }

    async fn find_all_paginated(&self, offset: i64, limit: i64) -> AppResult<(Vec<Music>, i64)> {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM musics")
            .fetch_one(&self.pool)
            .await?;
        let rows = sqlx::query_as::<_, MusicRow>("SELECT * FROM musics ORDER BY id OFFSET $1 LIMIT $2")
            .bind(offset)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;
        Ok((rows.into_iter().map(MusicRow::into_entity).collect(), total))
    }

    async fn find_by_id(&self, id: i64) -> AppResult<Music> {
        let row = sqlx::query_as::<_, MusicRow>("SELECT * FROM musics WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("music not found".to_string()))?;
        Ok(row.into_entity())
    }

    async fn find_by_ids(&self, ids: &[i64]) -> AppResult<Vec<Music>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let rows = sqlx::query_as::<_, MusicRow>("SELECT * FROM musics WHERE id = ANY($1)")
            .bind(ids)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(MusicRow::into_entity).collect())
    }

    async fn find_by_user_id(&self, user_id: i64) -> AppResult<Vec<Music>> {
        let rows = sqlx::query_as::<_, MusicRow>("SELECT * FROM musics WHERE user_id = $1 ORDER BY id")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(MusicRow::into_entity).collect())
    }

    async fn update(&self, music: &Music) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE musics SET
                title = $1, artist = $2, album = $3, genre = $4, duration = $5, bpm = $6,
                key = $7, cover = $8, lyrics = $9, description = $10, user_id = $11, updated_at = NOW()
            WHERE id = $12
            "#,
        )
        .bind(&music.title)
        .bind(&music.artist)
        .bind(&music.album)
        .bind(&music.genre)
        .bind(music.duration)
        .bind(music.bpm)
        .bind(&music.key)
        .bind(&music.cover)
        .bind(&music.lyrics)
        .bind(&music.description)
        .bind(music.user_id)
        .bind(music.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: i64) -> AppResult<()> {
        sqlx::query("DELETE FROM musics WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
