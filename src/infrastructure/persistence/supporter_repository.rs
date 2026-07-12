//! Supporter repository (sqlx/PostgreSQL), preloading the related user.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::Supporter;
use crate::domain::repository::SupporterRepository;
use crate::error::{AppError, AppResult};
use crate::infrastructure::persistence::rows::{SupporterRow, UserRow};

pub struct PgSupporterRepository {
    pool: PgPool,
}

impl PgSupporterRepository {
    pub fn new(pool: PgPool) -> Self {
        PgSupporterRepository { pool }
    }

    fn limit_offset(limit: i64, offset: i64) -> String {
        if limit > 0 {
            format!(" LIMIT {limit} OFFSET {offset}")
        } else {
            String::new()
        }
    }

    async fn attach_user(&self, supporter: &mut Supporter) -> AppResult<()> {
        if let Some(user_id) = supporter.user_id {
            if let Some(row) = sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?
            {
                supporter.user = Some(row.into_entity());
            }
        }
        Ok(())
    }

    async fn attach_many(&self, supporters: &mut [Supporter]) -> AppResult<()> {
        for supporter in supporters.iter_mut() {
            self.attach_user(supporter).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl SupporterRepository for PgSupporterRepository {
    async fn create(&self, supporter: &mut Supporter) -> AppResult<()> {
        let rec: (i64, DateTime<Utc>, DateTime<Utc>) = sqlx::query_as(
            r#"
            INSERT INTO supporters
                (name, email, phone, type, website, address, logo, description, user_id,
                 created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9, NOW(), NOW())
            RETURNING id, created_at, updated_at
            "#,
        )
        .bind(&supporter.name)
        .bind(&supporter.email)
        .bind(&supporter.phone)
        .bind(&supporter.r#type)
        .bind(&supporter.website)
        .bind(&supporter.address)
        .bind(&supporter.logo)
        .bind(&supporter.description)
        .bind(supporter.user_id)
        .fetch_one(&self.pool)
        .await?;
        supporter.id = rec.0;
        supporter.created_at = rec.1;
        supporter.updated_at = rec.2;
        Ok(())
    }

    async fn find_by_id(&self, id: i64) -> AppResult<Supporter> {
        let row = sqlx::query_as::<_, SupporterRow>("SELECT * FROM supporters WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("supporter not found".to_string()))?;
        let mut supporter = row.into_entity();
        self.attach_user(&mut supporter).await?;
        Ok(supporter)
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Supporter> {
        let row = sqlx::query_as::<_, SupporterRow>("SELECT * FROM supporters WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("supporter not found".to_string()))?;
        let mut supporter = row.into_entity();
        self.attach_user(&mut supporter).await?;
        Ok(supporter)
    }

    async fn find_all(&self, limit: i64, offset: i64) -> AppResult<Vec<Supporter>> {
        let sql = format!(
            "SELECT * FROM supporters ORDER BY created_at DESC{}",
            Self::limit_offset(limit, offset)
        );
        let rows = sqlx::query_as::<_, SupporterRow>(&sql).fetch_all(&self.pool).await?;
        let mut supporters: Vec<Supporter> = rows.into_iter().map(SupporterRow::into_entity).collect();
        self.attach_many(&mut supporters).await?;
        Ok(supporters)
    }

    async fn find_by_type(&self, supporter_type: &str, limit: i64, offset: i64) -> AppResult<Vec<Supporter>> {
        let sql = format!(
            "SELECT * FROM supporters WHERE type = $1 ORDER BY created_at DESC{}",
            Self::limit_offset(limit, offset)
        );
        let rows = sqlx::query_as::<_, SupporterRow>(&sql)
            .bind(supporter_type)
            .fetch_all(&self.pool)
            .await?;
        let mut supporters: Vec<Supporter> = rows.into_iter().map(SupporterRow::into_entity).collect();
        self.attach_many(&mut supporters).await?;
        Ok(supporters)
    }

    async fn find_by_user(&self, user_id: i64, limit: i64, offset: i64) -> AppResult<Vec<Supporter>> {
        let sql = format!(
            "SELECT * FROM supporters WHERE user_id = $1 ORDER BY created_at DESC{}",
            Self::limit_offset(limit, offset)
        );
        let rows = sqlx::query_as::<_, SupporterRow>(&sql)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;
        let mut supporters: Vec<Supporter> = rows.into_iter().map(SupporterRow::into_entity).collect();
        self.attach_many(&mut supporters).await?;
        Ok(supporters)
    }

    async fn update(&self, supporter: &Supporter) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE supporters SET
                name = $1, email = $2, phone = $3, type = $4, website = $5, address = $6,
                logo = $7, description = $8, user_id = $9, updated_at = NOW()
            WHERE id = $10
            "#,
        )
        .bind(&supporter.name)
        .bind(&supporter.email)
        .bind(&supporter.phone)
        .bind(&supporter.r#type)
        .bind(&supporter.website)
        .bind(&supporter.address)
        .bind(&supporter.logo)
        .bind(&supporter.description)
        .bind(supporter.user_id)
        .bind(supporter.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: i64) -> AppResult<()> {
        sqlx::query("DELETE FROM supporters WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn count(&self) -> AppResult<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM supporters")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }

    async fn count_by_type(&self, supporter_type: &str) -> AppResult<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM supporters WHERE type = $1")
            .bind(supporter_type)
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }
}
