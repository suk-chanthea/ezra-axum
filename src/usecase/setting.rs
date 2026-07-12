//! Setting use case, mirroring Go `settingUseCase`.

use std::sync::Arc;

use crate::domain::repository::SettingRepository;
use crate::error::{AppError, AppResult};
use crate::interface::http::dto::request::UpdateSettingRequest;
use crate::interface::http::dto::response::SettingResponse;

pub struct SettingUseCase {
    setting_repo: Arc<dyn SettingRepository>,
}

impl SettingUseCase {
    pub fn new(setting_repo: Arc<dyn SettingRepository>) -> Self {
        SettingUseCase { setting_repo }
    }

    pub async fn get_user_settings(&self, user_id: i64) -> AppResult<SettingResponse> {
        let setting = self
            .setting_repo
            .find_by_user_id(user_id)
            .await
            .map_err(|_| AppError::NotFound("settings not found".to_string()))?;
        Ok(SettingResponse::from_entity(&setting))
    }

    pub async fn update_settings(
        &self,
        user_id: i64,
        req: UpdateSettingRequest,
    ) -> AppResult<SettingResponse> {
        let mut setting = self
            .setting_repo
            .find_by_user_id(user_id)
            .await
            .map_err(|_| AppError::NotFound("settings not found".to_string()))?;

        setting.language = req.language;
        setting.theme = req.theme;
        setting.notify_on_booking = req.notify_on_booking;
        setting.notify_on_music = req.notify_on_music;
        setting.notify_on_event = req.notify_on_event;
        setting.enable_push_notifications = req.enable_push_notifications;

        if !setting.is_valid() {
            return Err(AppError::BadRequest("invalid settings data".to_string()));
        }

        self.setting_repo.update(&mut setting).await?;
        Ok(SettingResponse::from_entity(&setting))
    }

    pub async fn reset_to_defaults(&self, user_id: i64) -> AppResult<SettingResponse> {
        let mut setting = self
            .setting_repo
            .find_by_user_id(user_id)
            .await
            .map_err(|_| AppError::NotFound("settings not found".to_string()))?;

        setting.language = "en".to_string();
        setting.theme = "light".to_string();
        setting.notify_on_booking = true;
        setting.notify_on_music = false;
        setting.notify_on_event = true;
        setting.enable_push_notifications = true;

        self.setting_repo.update(&mut setting).await?;
        Ok(SettingResponse::from_entity(&setting))
    }
}
