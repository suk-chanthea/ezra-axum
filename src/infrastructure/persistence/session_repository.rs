//! Session repository (sqlx/PostgreSQL).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::Session;
use crate::domain::repository::SessionRepository;
use crate::error::AppResult;

pub struct PgSessionRepository {
    pool: PgPool,
}

impl PgSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        PgSessionRepository { pool }
    }
}

#[async_trait]
impl SessionRepository for PgSessionRepository {
    async fn save(&self, session: &mut Session) -> AppResult<()> {
        let rec: (i64, DateTime<Utc>, DateTime<Utc>, DateTime<Utc>) = sqlx::query_as(
            r#"
            INSERT INTO sessions (user_id, device_id, device_name, token, expires_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
            RETURNING id, expires_at, created_at, updated_at
            "#,
        )
        .bind(session.user_id)
        .bind(&session.device_id)
        .bind(&session.device_name)
        .bind(&session.token)
        .bind(session.expires_at)
        .fetch_one(&self.pool)
        .await?;

        session.id = rec.0;
        session.expires_at = rec.1;
        session.created_at = rec.2;
        session.updated_at = rec.3;
        Ok(())
    }

    async fn verify(&self, user_id: i64, token: &str) -> AppResult<bool> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM sessions WHERE user_id = $1 AND token = $2 AND expires_at > NOW())"
        )
        .bind(user_id)
        .bind(token)
        .fetch_one(&self.pool)
        .await?;
        Ok(exists)
    }

    async fn find_by_user_id(&self, user_id: i64) -> AppResult<Vec<Session>> {
        let rows = sqlx::query_as::<_, (i64, i64, String, String, String, DateTime<Utc>, DateTime<Utc>, DateTime<Utc>)>(
            "SELECT id, user_id, device_id, device_name, token, expires_at, created_at, updated_at FROM sessions WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let sessions = rows.into_iter().map(|row| Session {
            id: row.0,
            user_id: row.1,
            device_id: row.2,
            device_name: row.3,
            token: row.4,
            expires_at: row.5,
            created_at: row.6,
            updated_at: row.7,
        }).collect();

        Ok(sessions)
    }

    async fn delete_by_id(&self, id: i64) -> AppResult<()> {
        sqlx::query("DELETE FROM sessions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete_by_token(&self, token: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM sessions WHERE token = $1")
            .bind(token)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete_by_user_id(&self, user_id: i64) -> AppResult<()> {
        sqlx::query("DELETE FROM sessions WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete_by_user_id_and_device(&self, user_id: i64, device_id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM sessions WHERE user_id = $1 AND device_id = $2")
            .bind(user_id)
            .bind(device_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
