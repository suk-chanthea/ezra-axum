//! Notification repository (sqlx/PostgreSQL).
//!
//! User-facing queries combine direct (user_id), band, and broadcast notifications while
//! excluding notifications the user created themselves, mirroring the Go implementation.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::Notification;
use crate::domain::repository::NotificationRepository;
use crate::error::{AppError, AppResult};
use crate::infrastructure::persistence::rows::NotificationRow;

pub struct PgNotificationRepository {
    pool: PgPool,
}

impl PgNotificationRepository {
    pub fn new(pool: PgPool) -> Self {
        PgNotificationRepository { pool }
    }

    async fn user_band_id(&self, user_id: i64) -> AppResult<Option<i64>> {
        let band_id: Option<Option<i64>> = sqlx::query_scalar("SELECT band_id FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(band_id.flatten())
    }

    /// Builds the visibility predicate: direct + band + broadcast, excluding own notifications.
    fn visibility_clause(band_id: Option<i64>) -> String {
        let band = match band_id {
            Some(b) => format!(" OR band_id = {b}"),
            None => String::new(),
        };
        format!("(user_id = $1{band} OR recipient_type = 'all') AND (sender_id IS NULL OR sender_id <> $1)")
    }
}

#[async_trait]
impl NotificationRepository for PgNotificationRepository {
    async fn create(&self, notification: &mut Notification) -> AppResult<()> {
        let rec: (i64, DateTime<Utc>, DateTime<Utc>) = sqlx::query_as(
            r#"
            INSERT INTO notifications
                (user_id, sender_id, band_id, recipient_type, title, message, type, related_type,
                 related_id, is_read, read_at, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11, NOW(), NOW())
            RETURNING id, created_at, updated_at
            "#,
        )
        .bind(notification.user_id)
        .bind(notification.sender_id)
        .bind(notification.band_id)
        .bind(&notification.recipient_type)
        .bind(&notification.title)
        .bind(&notification.message)
        .bind(&notification.r#type)
        .bind(&notification.related_type)
        .bind(notification.related_id)
        .bind(notification.is_read)
        .bind(notification.read_at)
        .fetch_one(&self.pool)
        .await?;
        notification.id = rec.0;
        notification.created_at = rec.1;
        notification.updated_at = rec.2;
        Ok(())
    }

    async fn find_by_id(&self, id: i64) -> AppResult<Notification> {
        let row = sqlx::query_as::<_, NotificationRow>("SELECT * FROM notifications WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("notification not found".to_string()))?;
        Ok(row.into_entity())
    }

    async fn find_by_user_id(&self, user_id: i64) -> AppResult<Vec<Notification>> {
        let band_id = self.user_band_id(user_id).await?;
        let sql = format!(
            "SELECT * FROM notifications WHERE {} ORDER BY created_at DESC",
            Self::visibility_clause(band_id)
        );
        let rows = sqlx::query_as::<_, NotificationRow>(&sql)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(NotificationRow::into_entity).collect())
    }

    async fn find_by_user_id_paginated(
        &self,
        user_id: i64,
        offset: i64,
        limit: i64,
    ) -> AppResult<(Vec<Notification>, i64)> {
        let band_id = self.user_band_id(user_id).await?;
        let clause = Self::visibility_clause(band_id);
        let count_sql = format!("SELECT COUNT(*) FROM notifications WHERE {clause}");
        let total: i64 = sqlx::query_scalar(&count_sql)
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;
        let sql = format!(
            "SELECT * FROM notifications WHERE {clause} ORDER BY created_at DESC OFFSET $2 LIMIT $3"
        );
        let rows = sqlx::query_as::<_, NotificationRow>(&sql)
            .bind(user_id)
            .bind(offset)
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;
        Ok((rows.into_iter().map(NotificationRow::into_entity).collect(), total))
    }

    async fn find_unread_by_user_id(&self, user_id: i64) -> AppResult<Vec<Notification>> {
        let band_id = self.user_band_id(user_id).await?;
        let sql = format!(
            "SELECT * FROM notifications WHERE ({}) AND is_read = FALSE ORDER BY created_at DESC",
            Self::visibility_clause(band_id)
        );
        let rows = sqlx::query_as::<_, NotificationRow>(&sql)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(NotificationRow::into_entity).collect())
    }

    async fn get_unread_count(&self, user_id: i64) -> AppResult<i64> {
        let band_id = self.user_band_id(user_id).await?;
        let sql = format!(
            "SELECT COUNT(*) FROM notifications WHERE ({}) AND is_read = FALSE",
            Self::visibility_clause(band_id)
        );
        let count: i64 = sqlx::query_scalar(&sql)
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }

    async fn find_by_band_id(&self, band_id: i64) -> AppResult<Vec<Notification>> {
        let rows = sqlx::query_as::<_, NotificationRow>(
            "SELECT * FROM notifications WHERE band_id = $1 ORDER BY created_at DESC",
        )
        .bind(band_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(NotificationRow::into_entity).collect())
    }

    async fn find_broadcast_notifications(&self) -> AppResult<Vec<Notification>> {
        let rows = sqlx::query_as::<_, NotificationRow>(
            "SELECT * FROM notifications WHERE recipient_type = 'all' ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(NotificationRow::into_entity).collect())
    }

    async fn update(&self, notification: &Notification) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE notifications SET
                user_id = $1, sender_id = $2, band_id = $3, recipient_type = $4, title = $5,
                message = $6, type = $7, related_type = $8, related_id = $9, is_read = $10,
                read_at = $11, updated_at = NOW()
            WHERE id = $12
            "#,
        )
        .bind(notification.user_id)
        .bind(notification.sender_id)
        .bind(notification.band_id)
        .bind(&notification.recipient_type)
        .bind(&notification.title)
        .bind(&notification.message)
        .bind(&notification.r#type)
        .bind(&notification.related_type)
        .bind(notification.related_id)
        .bind(notification.is_read)
        .bind(notification.read_at)
        .bind(notification.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: i64) -> AppResult<()> {
        sqlx::query("DELETE FROM notifications WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete_all_by_user_id(&self, user_id: i64) -> AppResult<()> {
        sqlx::query("DELETE FROM notifications WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn mark_as_read(&self, id: i64, _user_id: i64) -> AppResult<()> {
        sqlx::query("UPDATE notifications SET is_read = TRUE, read_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn mark_all_as_read(&self, user_id: i64) -> AppResult<()> {
        let band_id = self.user_band_id(user_id).await?;
        let band = match band_id {
            Some(b) => format!(" OR band_id = {b}"),
            None => String::new(),
        };
        let sql = format!(
            "UPDATE notifications SET is_read = TRUE, read_at = NOW() WHERE (user_id = $1{band} OR recipient_type = 'all') AND is_read = FALSE"
        );
        sqlx::query(&sql)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
