//! Favorite repository (sqlx/PostgreSQL).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::{Favorite, Music};
use crate::domain::repository::FavoriteRepository;
use crate::error::AppResult;
use crate::infrastructure::persistence::rows::MusicRow;

pub struct PgFavoriteRepository {
    pool: PgPool,
}

impl PgFavoriteRepository {
    pub fn new(pool: PgPool) -> Self {
        PgFavoriteRepository { pool }
    }
}

#[async_trait]
impl FavoriteRepository for PgFavoriteRepository {
    async fn create(&self, favorite: &mut Favorite) -> AppResult<()> {
        let rec: (i64, DateTime<Utc>) = sqlx::query_as(
            "INSERT INTO favorites (user_id, music_id, created_at) VALUES ($1,$2, NOW()) RETURNING id, created_at",
        )
        .bind(favorite.user_id)
        .bind(favorite.music_id)
        .fetch_one(&self.pool)
        .await?;
        favorite.id = rec.0;
        favorite.created_at = rec.1;
        Ok(())
    }

    async fn delete(&self, user_id: i64, music_id: i64) -> AppResult<()> {
        sqlx::query("DELETE FROM favorites WHERE user_id = $1 AND music_id = $2")
            .bind(user_id)
            .bind(music_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_by_user_id(&self, user_id: i64) -> AppResult<Vec<Music>> {
        let rows = sqlx::query_as::<_, MusicRow>(
            r#"
            SELECT m.* FROM musics m
            JOIN favorites f ON m.id = f.music_id
            WHERE f.user_id = $1
            ORDER BY f.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(MusicRow::into_entity).collect())
    }

    async fn get_by_user_id_paginated(
        &self,
        user_id: i64,
        offset: i64,
        limit: i64,
    ) -> AppResult<(Vec<Music>, i64)> {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM favorites WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;
        let rows = sqlx::query_as::<_, MusicRow>(
            r#"
            SELECT m.* FROM musics m
            JOIN favorites f ON m.id = f.music_id
            WHERE f.user_id = $1
            ORDER BY f.created_at DESC
            OFFSET $2 LIMIT $3
            "#,
        )
        .bind(user_id)
        .bind(offset)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok((rows.into_iter().map(MusicRow::into_entity).collect(), total))
    }

    async fn is_favorite(&self, user_id: i64, music_id: i64) -> AppResult<bool> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM favorites WHERE user_id = $1 AND music_id = $2",
        )
        .bind(user_id)
        .bind(music_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count > 0)
    }

    async fn get_favorite_count(&self, music_id: i64) -> AppResult<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM favorites WHERE music_id = $1")
            .bind(music_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }
}
