//! Church repository (sqlx/PostgreSQL), with owner preloading and member queries.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::{Church, User};
use crate::domain::repository::ChurchRepository;
use crate::error::{AppError, AppResult};
use crate::infrastructure::persistence::rows::{ChurchRow, UserRow};

pub struct PgChurchRepository {
    pool: PgPool,
}

impl PgChurchRepository {
    pub fn new(pool: PgPool) -> Self {
        PgChurchRepository { pool }
    }
}

#[async_trait]
impl ChurchRepository for PgChurchRepository {
    async fn create(&self, church: &mut Church) -> AppResult<()> {
        let rec: (i64, DateTime<Utc>, DateTime<Utc>) = sqlx::query_as(
            r#"
            INSERT INTO churches
                (name, address, phone, email, website, pastor_name, description, logo,
                 established_date, denomination, owner_id, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11, NOW(), NOW())
            RETURNING id, created_at, updated_at
            "#,
        )
        .bind(&church.name)
        .bind(&church.address)
        .bind(&church.phone)
        .bind(&church.email)
        .bind(&church.website)
        .bind(&church.pastor_name)
        .bind(&church.description)
        .bind(&church.logo)
        .bind(church.established_date)
        .bind(&church.denomination)
        .bind(church.owner_id)
        .fetch_one(&self.pool)
        .await?;
        church.id = rec.0;
        church.created_at = rec.1;
        church.updated_at = rec.2;
        Ok(())
    }

    async fn find_by_id(&self, id: i64) -> AppResult<Church> {
        let row = sqlx::query_as::<_, ChurchRow>("SELECT * FROM churches WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("church not found".to_string()))?;
        let mut church = row.into_entity();
        if let Some(owner_id) = church.owner_id {
            if let Some(user_row) = sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE id = $1")
                .bind(owner_id)
                .fetch_optional(&self.pool)
                .await?
            {
                church.owner = Some(user_row.into_entity());
            }
        }
        Ok(church)
    }

    async fn find_by_name(&self, name: &str) -> AppResult<Church> {
        let row = sqlx::query_as::<_, ChurchRow>("SELECT * FROM churches WHERE name = $1")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("church not found".to_string()))?;
        Ok(row.into_entity())
    }

    async fn find_all(&self, limit: i64, offset: i64) -> AppResult<Vec<Church>> {
        let rows = if limit > 0 {
            sqlx::query_as::<_, ChurchRow>(
                "SELECT * FROM churches ORDER BY name ASC LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, ChurchRow>("SELECT * FROM churches ORDER BY name ASC")
                .fetch_all(&self.pool)
                .await?
        };
        Ok(rows.into_iter().map(ChurchRow::into_entity).collect())
    }

    async fn find_by_denomination(&self, denomination: &str, limit: i64, offset: i64) -> AppResult<Vec<Church>> {
        let rows = if limit > 0 {
            sqlx::query_as::<_, ChurchRow>(
                "SELECT * FROM churches WHERE denomination = $1 ORDER BY name ASC LIMIT $2 OFFSET $3",
            )
            .bind(denomination)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, ChurchRow>(
                "SELECT * FROM churches WHERE denomination = $1 ORDER BY name ASC",
            )
            .bind(denomination)
            .fetch_all(&self.pool)
            .await?
        };
        Ok(rows.into_iter().map(ChurchRow::into_entity).collect())
    }

    async fn update(&self, church: &Church) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE churches SET
                name = $1, address = $2, phone = $3, email = $4, website = $5,
                pastor_name = $6, description = $7, logo = $8, established_date = $9,
                denomination = $10, owner_id = $11, updated_at = NOW()
            WHERE id = $12
            "#,
        )
        .bind(&church.name)
        .bind(&church.address)
        .bind(&church.phone)
        .bind(&church.email)
        .bind(&church.website)
        .bind(&church.pastor_name)
        .bind(&church.description)
        .bind(&church.logo)
        .bind(church.established_date)
        .bind(&church.denomination)
        .bind(church.owner_id)
        .bind(church.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: i64) -> AppResult<()> {
        sqlx::query("DELETE FROM churches WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn count(&self) -> AppResult<i64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM churches")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }

    async fn count_members(&self, church_id: i64, status: &str) -> AppResult<i64> {
        let count: i64 = if status.is_empty() {
            sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE church_id = $1")
                .bind(church_id)
                .fetch_one(&self.pool)
                .await?
        } else {
            sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE church_id = $1 AND church_status = $2")
                .bind(church_id)
                .bind(status)
                .fetch_one(&self.pool)
                .await?
        };
        Ok(count)
    }

    async fn find_members(&self, church_id: i64, status: &str, limit: i64, offset: i64) -> AppResult<Vec<User>> {
        let rows = match (status.is_empty(), limit > 0) {
            (true, true) => {
                sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE church_id = $1 LIMIT $2 OFFSET $3")
                    .bind(church_id)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(&self.pool)
                    .await?
            }
            (true, false) => {
                sqlx::query_as::<_, UserRow>("SELECT * FROM users WHERE church_id = $1")
                    .bind(church_id)
                    .fetch_all(&self.pool)
                    .await?
            }
            (false, true) => {
                sqlx::query_as::<_, UserRow>(
                    "SELECT * FROM users WHERE church_id = $1 AND church_status = $2 LIMIT $3 OFFSET $4",
                )
                .bind(church_id)
                .bind(status)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            (false, false) => {
                sqlx::query_as::<_, UserRow>(
                    "SELECT * FROM users WHERE church_id = $1 AND church_status = $2",
                )
                .bind(church_id)
                .bind(status)
                .fetch_all(&self.pool)
                .await?
            }
        };
        Ok(rows.into_iter().map(UserRow::into_entity).collect())
    }
}
