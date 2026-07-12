//! Band repository (sqlx/PostgreSQL), including band_musics join and member queries.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::{Band, Music, User};
use crate::domain::repository::BandRepository;
use crate::error::{AppError, AppResult};
use crate::infrastructure::persistence::rows::{BandRow, MusicRow, UserRow};

pub struct PgBandRepository {
    pool: PgPool,
}

impl PgBandRepository {
    pub fn new(pool: PgPool) -> Self {
        PgBandRepository { pool }
    }
}

#[async_trait]
impl BandRepository for PgBandRepository {
    async fn save(&self, band: &mut Band) -> AppResult<()> {
        let rec: (i64, DateTime<Utc>, DateTime<Utc>) = sqlx::query_as(
            r#"
            INSERT INTO bands (name, description, cover, is_public, user_id, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5, NOW(), NOW())
            RETURNING id, created_at, updated_at
            "#,
        )
        .bind(&band.name)
        .bind(&band.description)
        .bind(&band.cover)
        .bind(band.is_public)
        .bind(band.user_id)
        .fetch_one(&self.pool)
        .await?;
        band.id = rec.0;
        band.created_at = rec.1;
        band.updated_at = rec.2;
        Ok(())
    }

    async fn find_all(&self) -> AppResult<Vec<Band>> {
        let rows = sqlx::query_as::<_, BandRow>("SELECT * FROM bands ORDER BY id")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(BandRow::into_entity).collect())
    }

    async fn find_all_paginated(&self, offset: i64, limit: i64) -> AppResult<(Vec<Band>, i64)> {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bands")
            .fetch_one(&self.pool)
            .await?;
        let rows = sqlx::query_as::<_, BandRow>("SELECT * FROM bands ORDER BY id OFFSET $1 LIMIT $2")
            .bind(offset)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;
        Ok((rows.into_iter().map(BandRow::into_entity).collect(), total))
    }

    async fn find_by_id(&self, id: i64) -> AppResult<Band> {
        let row = sqlx::query_as::<_, BandRow>("SELECT * FROM bands WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("band not found".to_string()))?;
        Ok(row.into_entity())
    }

    async fn find_by_user_id(&self, user_id: i64) -> AppResult<Vec<Band>> {
        let rows = sqlx::query_as::<_, BandRow>("SELECT * FROM bands WHERE user_id = $1 ORDER BY id")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(BandRow::into_entity).collect())
    }

    async fn find_by_user_id_paginated(
        &self,
        user_id: i64,
        offset: i64,
        limit: i64,
    ) -> AppResult<(Vec<Band>, i64)> {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bands WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;
        let rows = sqlx::query_as::<_, BandRow>(
            "SELECT * FROM bands WHERE user_id = $1 ORDER BY id OFFSET $2 LIMIT $3",
        )
        .bind(user_id)
        .bind(offset)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok((rows.into_iter().map(BandRow::into_entity).collect(), total))
    }

    async fn find_public_bands(&self) -> AppResult<Vec<Band>> {
        let rows = sqlx::query_as::<_, BandRow>("SELECT * FROM bands WHERE is_public = TRUE ORDER BY id")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(BandRow::into_entity).collect())
    }

    async fn find_public_bands_paginated(&self, offset: i64, limit: i64) -> AppResult<(Vec<Band>, i64)> {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bands WHERE is_public = TRUE")
            .fetch_one(&self.pool)
            .await?;
        let rows = sqlx::query_as::<_, BandRow>(
            "SELECT * FROM bands WHERE is_public = TRUE ORDER BY id OFFSET $1 LIMIT $2",
        )
        .bind(offset)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok((rows.into_iter().map(BandRow::into_entity).collect(), total))
    }

    async fn update(&self, band: &Band) -> AppResult<()> {
        sqlx::query(
            "UPDATE bands SET name = $1, description = $2, cover = $3, is_public = $4, user_id = $5, updated_at = NOW() WHERE id = $6",
        )
        .bind(&band.name)
        .bind(&band.description)
        .bind(&band.cover)
        .bind(band.is_public)
        .bind(band.user_id)
        .bind(band.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: i64) -> AppResult<()> {
        sqlx::query("DELETE FROM bands WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn add_musics_to_band(&self, band_id: i64, music_ids: &[i64]) -> AppResult<()> {
        for music_id in music_ids {
            let exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM band_musics WHERE band_id = $1 AND music_id = $2",
            )
            .bind(band_id)
            .bind(music_id)
            .fetch_one(&self.pool)
            .await?;
            if exists == 0 {
                sqlx::query(
                    "INSERT INTO band_musics (band_id, music_id, display_order, created_at) VALUES ($1,$2,0, NOW())",
                )
                .bind(band_id)
                .bind(music_id)
                .execute(&self.pool)
                .await?;
            }
        }
        Ok(())
    }

    async fn remove_music_from_band(&self, band_id: i64, music_id: i64) -> AppResult<()> {
        sqlx::query("DELETE FROM band_musics WHERE band_id = $1 AND music_id = $2")
            .bind(band_id)
            .bind(music_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_band_musics(&self, band_id: i64) -> AppResult<Vec<Music>> {
        let rows = sqlx::query_as::<_, MusicRow>(
            r#"
            SELECT m.* FROM musics m
            JOIN band_musics bm ON m.id = bm.music_id
            WHERE bm.band_id = $1
            ORDER BY bm.display_order ASC, bm.created_at ASC
            "#,
        )
        .bind(band_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(MusicRow::into_entity).collect())
    }

    async fn reorder_band_musics(&self, band_id: i64, music_orders: &[(i64, i32)]) -> AppResult<()> {
        for (music_id, order) in music_orders {
            sqlx::query("UPDATE band_musics SET display_order = $1 WHERE band_id = $2 AND music_id = $3")
                .bind(order)
                .bind(band_id)
                .bind(music_id)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    async fn get_band_member_count(&self, band_id: i64) -> AppResult<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE band_id = $1")
            .bind(band_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }

    async fn get_band_members(&self, band_id: i64) -> AppResult<Vec<User>> {
        let rows = sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE band_id = $1 ORDER BY id")
            .bind(band_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(UserRow::into_entity).collect())
    }
}
