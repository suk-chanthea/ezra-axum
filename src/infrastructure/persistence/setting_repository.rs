//! Setting repository (sqlx/PostgreSQL).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::domain::entity::Setting;
use crate::domain::repository::SettingRepository;
use crate::error::{AppError, AppResult};
use crate::infrastructure::persistence::rows::SettingRow;

pub struct PgSettingRepository {
    pool: PgPool,
}

impl PgSettingRepository {
    pub fn new(pool: PgPool) -> Self {
        PgSettingRepository { pool }
    }
}

#[async_trait]
impl SettingRepository for PgSettingRepository {
    async fn save(&self, setting: &mut Setting) -> AppResult<()> {
        let rec: (i64, DateTime<Utc>, DateTime<Utc>) = sqlx::query_as(
            r#"
            INSERT INTO settings
                (user_id, language, theme, notify_on_booking, notify_on_music, notify_on_event,
                 enable_push_notifications, created_at, updated_at)
            VALUES ($1,$2,$3,$4,$5,$6,$7, NOW(), NOW())
            RETURNING id, created_at, updated_at
            "#,
        )
        .bind(setting.user_id)
        .bind(&setting.language)
        .bind(&setting.theme)
        .bind(setting.notify_on_booking)
        .bind(setting.notify_on_music)
        .bind(setting.notify_on_event)
        .bind(setting.enable_push_notifications)
        .fetch_one(&self.pool)
        .await?;
        setting.id = rec.0;
        setting.created_at = rec.1;
        setting.updated_at = rec.2;
        Ok(())
    }

    async fn find_by_user_id(&self, user_id: i64) -> AppResult<Setting> {
        let row = sqlx::query_as::<_, SettingRow>("SELECT * FROM settings WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("settings not found".to_string()))?;
        Ok(row.into_entity())
    }

    async fn update(&self, setting: &mut Setting) -> AppResult<()> {
        let rec: (DateTime<Utc>,) = sqlx::query_as(
            r#"
            UPDATE settings SET
                language = $1, theme = $2, notify_on_booking = $3, notify_on_music = $4,
                notify_on_event = $5, enable_push_notifications = $6, updated_at = NOW()
            WHERE user_id = $7
            RETURNING updated_at
            "#,
        )
        .bind(&setting.language)
        .bind(&setting.theme)
        .bind(setting.notify_on_booking)
        .bind(setting.notify_on_music)
        .bind(setting.notify_on_event)
        .bind(setting.enable_push_notifications)
        .bind(setting.user_id)
        .fetch_one(&self.pool)
        .await?;
        setting.updated_at = rec.0;
        Ok(())
    }

    async fn delete(&self, user_id: i64) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM settings WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("settings not found".to_string()));
        }
        Ok(())
    }
}
