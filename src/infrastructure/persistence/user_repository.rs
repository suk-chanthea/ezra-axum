//! User repository (sqlx/PostgreSQL).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::User;
use crate::domain::repository::UserRepository;
use crate::error::{AppError, AppResult};
use crate::infrastructure::persistence::rows::UserRow;

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        PgUserRepository { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn save(&self, user: &mut User) -> AppResult<()> {
        let rec: (i64, DateTime<Utc>, DateTime<Utc>) = sqlx::query_as(
            r#"
            INSERT INTO users
                (username, name, profile, email, email_verified, phone, password, role,
                 token, provider, provider_id, birthday, church_id, church_status, band_id, bio,
                 created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16, NOW(), NOW())
            RETURNING id, created_at, updated_at
            "#,
        )
        .bind(&user.username)
        .bind(&user.name)
        .bind(&user.profile)
        .bind(&user.email)
        .bind(user.email_verified)
        .bind(&user.phone)
        .bind(&user.password)
        .bind(&user.role)
        .bind(&user.token)
        .bind(&user.provider)
        .bind(&user.provider_id)
        .bind(user.birthday)
        .bind(user.church_id)
        .bind(&user.church_status)
        .bind(user.band_id)
        .bind(&user.bio)
        .fetch_one(&self.pool)
        .await?;

        user.id = rec.0;
        user.created_at = rec.1;
        user.updated_at = rec.2;
        Ok(())
    }

    async fn find_by_username(&self, username: &str) -> AppResult<User> {
        let row = sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;
        Ok(row.into_entity())
    }

    async fn find_by_email(&self, email: &str) -> AppResult<User> {
        let row = sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;
        Ok(row.into_entity())
    }

    async fn find_by_id(&self, id: i64) -> AppResult<User> {
        let row = sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;
        Ok(row.into_entity())
    }

    async fn find_by_provider_id(&self, provider: &str, provider_id: &str) -> AppResult<User> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT * FROM users WHERE provider = $1 AND provider_id = $2",
        )
        .bind(provider)
        .bind(provider_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;
        Ok(row.into_entity())
    }

    async fn update(&self, user: &User) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE users SET
                username = $1, name = $2, profile = $3, email = $4, email_verified = $5,
                phone = $6, password = $7, role = $8, token = $9, provider = $10,
                provider_id = $11, birthday = $12, church_id = $13, church_status = $14,
                band_id = $15, bio = $16, updated_at = NOW()
            WHERE id = $17
            "#,
        )
        .bind(&user.username)
        .bind(&user.name)
        .bind(&user.profile)
        .bind(&user.email)
        .bind(user.email_verified)
        .bind(&user.phone)
        .bind(&user.password)
        .bind(&user.role)
        .bind(&user.token)
        .bind(&user.provider)
        .bind(&user.provider_id)
        .bind(user.birthday)
        .bind(user.church_id)
        .bind(&user.church_status)
        .bind(user.band_id)
        .bind(&user.bio)
        .bind(user.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update_token(&self, id: i64, token: &str) -> AppResult<()> {
        sqlx::query("UPDATE users SET token = $1, updated_at = NOW() WHERE id = $2")
            .bind(token)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete(&self, id: i64) -> AppResult<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
