//! OTP repository (sqlx/PostgreSQL).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::Otp;
use crate::domain::repository::OtpRepository;
use crate::error::{AppError, AppResult};
use crate::infrastructure::persistence::rows::OtpRow;

pub struct PgOtpRepository {
    pool: PgPool,
}

impl PgOtpRepository {
    pub fn new(pool: PgPool) -> Self {
        PgOtpRepository { pool }
    }
}

#[async_trait]
impl OtpRepository for PgOtpRepository {
    async fn save(&self, otp: &mut Otp) -> AppResult<()> {
        let rec: (i64, DateTime<Utc>, DateTime<Utc>) = sqlx::query_as(
            r#"
            INSERT INTO otps (email, code, purpose, expires_at, verified, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5, NOW(), NOW())
            RETURNING id, created_at, updated_at
            "#,
        )
        .bind(&otp.email)
        .bind(&otp.code)
        .bind(&otp.purpose)
        .bind(otp.expires_at)
        .bind(otp.verified)
        .fetch_one(&self.pool)
        .await?;
        otp.id = rec.0;
        otp.created_at = rec.1;
        otp.updated_at = rec.2;
        Ok(())
    }

    async fn find_by_email_and_purpose(&self, email: &str, purpose: &str) -> AppResult<Otp> {
        let row = sqlx::query_as::<_, OtpRow>(
            r#"
            SELECT * FROM otps
            WHERE email = $1 AND purpose = $2 AND verified = FALSE AND expires_at > NOW()
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(email)
        .bind(purpose)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("otp not found".to_string()))?;
        Ok(row.into_entity())
    }

    async fn find_by_email_code_and_purpose(&self, email: &str, code: &str, purpose: &str) -> AppResult<Otp> {
        let row = sqlx::query_as::<_, OtpRow>(
            r#"
            SELECT * FROM otps
            WHERE email = $1 AND code = $2 AND purpose = $3
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(email)
        .bind(code)
        .bind(purpose)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("otp not found".to_string()))?;
        Ok(row.into_entity())
    }

    async fn update(&self, otp: &Otp) -> AppResult<()> {
        sqlx::query(
            "UPDATE otps SET email = $1, code = $2, purpose = $3, expires_at = $4, verified = $5, updated_at = NOW() WHERE id = $6",
        )
        .bind(&otp.email)
        .bind(&otp.code)
        .bind(&otp.purpose)
        .bind(otp.expires_at)
        .bind(otp.verified)
        .bind(otp.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete_by_email(&self, email: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM otps WHERE email = $1")
            .bind(email)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete_by_email_and_purpose(&self, email: &str, purpose: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM otps WHERE email = $1 AND purpose = $2")
            .bind(email)
            .bind(purpose)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete_expired(&self) -> AppResult<()> {
        sqlx::query("DELETE FROM otps WHERE expires_at < NOW()")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
