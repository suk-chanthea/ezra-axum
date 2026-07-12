//! Device token repository (sqlx/PostgreSQL), with upsert-on-save semantics.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::DeviceToken;
use crate::domain::repository::DeviceTokenRepository;
use crate::error::AppResult;
use crate::infrastructure::persistence::rows::DeviceTokenRow;

pub struct PgDeviceTokenRepository {
    pool: PgPool,
}

impl PgDeviceTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        PgDeviceTokenRepository { pool }
    }
}

#[async_trait]
impl DeviceTokenRepository for PgDeviceTokenRepository {
    async fn save(&self, token: &mut DeviceToken) -> AppResult<()> {
        // Upsert based on (user_id, token): reactivate and update platform if it exists.
        if let Some(existing) = sqlx::query_as::<_, DeviceTokenRow>(
            "SELECT * FROM device_tokens WHERE user_id = $1 AND token = $2",
        )
        .bind(token.user_id)
        .bind(&token.token)
        .fetch_optional(&self.pool)
        .await?
        {
            let rec: (DateTime<Utc>,) = sqlx::query_as(
                "UPDATE device_tokens SET platform = $1, is_active = TRUE, updated_at = NOW() WHERE id = $2 RETURNING updated_at",
            )
            .bind(&token.platform)
            .bind(existing.id)
            .fetch_one(&self.pool)
            .await?;
            token.id = existing.id;
            token.created_at = existing.created_at;
            token.updated_at = rec.0;
            return Ok(());
        }

        let rec: (i64, DateTime<Utc>, DateTime<Utc>) = sqlx::query_as(
            r#"
            INSERT INTO device_tokens (user_id, token, platform, is_active, created_at, updated_at)
            VALUES ($1,$2,$3,$4, NOW(), NOW())
            RETURNING id, created_at, updated_at
            "#,
        )
        .bind(token.user_id)
        .bind(&token.token)
        .bind(&token.platform)
        .bind(token.is_active)
        .fetch_one(&self.pool)
        .await?;
        token.id = rec.0;
        token.created_at = rec.1;
        token.updated_at = rec.2;
        Ok(())
    }

    async fn get_active_tokens_by_user_id(&self, user_id: i64) -> AppResult<Vec<String>> {
        let tokens: Vec<String> = sqlx::query_scalar(
            "SELECT token FROM device_tokens WHERE user_id = $1 AND is_active = TRUE",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(tokens)
    }

    async fn get_tokens_by_band_id(&self, band_id: i64) -> AppResult<Vec<String>> {
        let tokens: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT dt.token FROM device_tokens dt
            JOIN users u ON u.id = dt.user_id
            WHERE u.band_id = $1 AND dt.is_active = TRUE
            "#,
        )
        .bind(band_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(tokens)
    }

    async fn get_all_active_tokens(&self) -> AppResult<Vec<String>> {
        let tokens: Vec<String> = sqlx::query_scalar("SELECT token FROM device_tokens WHERE is_active = TRUE")
            .fetch_all(&self.pool)
            .await?;
        Ok(tokens)
    }

    async fn get_all_active_tokens_except(&self, exclude_user_id: i64) -> AppResult<Vec<String>> {
        let tokens: Vec<String> = sqlx::query_scalar(
            "SELECT token FROM device_tokens WHERE is_active = TRUE AND user_id <> $1",
        )
        .bind(exclude_user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(tokens)
    }

    async fn delete_token(&self, token: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM device_tokens WHERE token = $1")
            .bind(token)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete_tokens(&self, tokens: &[String]) -> AppResult<()> {
        if tokens.is_empty() {
            return Ok(());
        }
        sqlx::query("DELETE FROM device_tokens WHERE token = ANY($1)")
            .bind(tokens)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn deactivate_token(&self, token: &str) -> AppResult<()> {
        sqlx::query("UPDATE device_tokens SET is_active = FALSE, updated_at = NOW() WHERE token = $1")
            .bind(token)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete_user_tokens(&self, user_id: i64) -> AppResult<()> {
        sqlx::query("DELETE FROM device_tokens WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
