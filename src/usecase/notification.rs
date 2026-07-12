//! Notification use case, mirroring Go `notificationUseCase`.

use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::entity::Notification;
use crate::domain::repository::NotificationRepository;
use crate::error::{AppError, AppResult};
use crate::infrastructure::fcm::FcmService;
use crate::interface::http::dto::request::CreateNotificationRequest;
use crate::interface::http::dto::response::{NotificationResponse, PaginationMetadata};

pub struct NotificationUseCase {
    notification_repo: Arc<dyn NotificationRepository>,
    fcm_service: Arc<dyn FcmService>,
}

impl NotificationUseCase {
    pub fn new(
        notification_repo: Arc<dyn NotificationRepository>,
        fcm_service: Arc<dyn FcmService>,
    ) -> Self {
        NotificationUseCase { notification_repo, fcm_service }
    }

    fn build_fcm_data(notification: &Notification) -> HashMap<String, String> {
        let mut data = HashMap::new();
        data.insert("notification_id".to_string(), notification.id.to_string());
        data.insert("type".to_string(), notification.r#type.clone());
        data.insert("recipient_type".to_string(), notification.recipient_type.clone());
        if !notification.related_type.is_empty() {
            data.insert("related_type".to_string(), notification.related_type.clone());
        }
        if let Some(rid) = notification.related_id {
            data.insert("related_id".to_string(), rid.to_string());
        }
        if let Some(sid) = notification.sender_id {
            data.insert("sender_id".to_string(), sid.to_string());
        }
        data
    }

    fn apply_related(notification: &mut Notification, req: &CreateNotificationRequest) {
        if !req.related_type.is_empty() {
            notification.related_type = req.related_type.clone();
        }
        if req.related_id.is_some() {
            notification.related_id = req.related_id;
        }
    }

    pub async fn create_notification(
        &self,
        sender_id: i64,
        req: CreateNotificationRequest,
    ) -> AppResult<NotificationResponse> {
        let user_id = req
            .user_id
            .ok_or_else(|| AppError::BadRequest("user_id is required for user notifications".to_string()))?;

        let mut notification =
            Notification::new_user(user_id, req.title.clone(), req.message.clone(), req.r#type.clone());
        notification.sender_id = Some(sender_id);
        Self::apply_related(&mut notification, &req);

        if !notification.is_valid() {
            return Err(AppError::BadRequest("invalid notification data".to_string()));
        }

        self.notification_repo.create(&mut notification).await?;

        let data = Self::build_fcm_data(&notification);
        let _ = self
            .fcm_service
            .send_to_user(user_id, &notification.title, &notification.message, data)
            .await;

        Ok(NotificationResponse::from_entity(&notification))
    }

    pub async fn create_band_notification(
        &self,
        sender_id: i64,
        band_id: i64,
        req: CreateNotificationRequest,
    ) -> AppResult<NotificationResponse> {
        let mut notification =
            Notification::new_band(band_id, req.title.clone(), req.message.clone(), req.r#type.clone());
        notification.sender_id = Some(sender_id);
        Self::apply_related(&mut notification, &req);

        if !notification.is_valid() {
            return Err(AppError::BadRequest("invalid notification data".to_string()));
        }

        self.notification_repo.create(&mut notification).await?;

        let data = Self::build_fcm_data(&notification);
        let _ = self
            .fcm_service
            .send_to_band(band_id, &notification.title, &notification.message, data)
            .await;

        Ok(NotificationResponse::from_entity(&notification))
    }

    pub async fn create_broadcast_notification(
        &self,
        sender_id: i64,
        req: CreateNotificationRequest,
    ) -> AppResult<NotificationResponse> {
        let mut notification =
            Notification::new_broadcast(req.title.clone(), req.message.clone(), req.r#type.clone());
        notification.sender_id = Some(sender_id);
        Self::apply_related(&mut notification, &req);

        if !notification.is_valid() {
            return Err(AppError::BadRequest("invalid notification data".to_string()));
        }

        self.notification_repo.create(&mut notification).await?;

        let data = Self::build_fcm_data(&notification);
        let _ = self
            .fcm_service
            .send_to_all_except(sender_id, &notification.title, &notification.message, data)
            .await;

        Ok(NotificationResponse::from_entity(&notification))
    }

    pub async fn get_notifications(
        &self,
        user_id: i64,
        page: i64,
        page_size: i64,
    ) -> AppResult<(Vec<NotificationResponse>, PaginationMetadata)> {
        let offset = (page - 1) * page_size;
        let (notifications, total) = self
            .notification_repo
            .find_by_user_id_paginated(user_id, offset, page_size)
            .await?;
        Ok((NotificationResponse::list(&notifications), PaginationMetadata::new(page, page_size, total)))
    }

    pub async fn get_unread_notifications(&self, user_id: i64) -> AppResult<Vec<NotificationResponse>> {
        let notifications = self.notification_repo.find_unread_by_user_id(user_id).await?;
        Ok(NotificationResponse::list(&notifications))
    }

    pub async fn get_unread_count(&self, user_id: i64) -> AppResult<i64> {
        self.notification_repo.get_unread_count(user_id).await
    }

    pub async fn get_notification_by_id(
        &self,
        user_id: i64,
        notif_id: i64,
    ) -> AppResult<NotificationResponse> {
        let notification = self
            .notification_repo
            .find_by_id(notif_id)
            .await
            .map_err(|_| AppError::NotFound("notification not found".to_string()))?;

        if notification.recipient_type == "user"
            && notification.user_id.map(|v| v != user_id).unwrap_or(true)
        {
            return Err(AppError::NotFound("unauthorized".to_string()));
        }

        Ok(NotificationResponse::from_entity(&notification))
    }

    pub async fn mark_as_read(&self, user_id: i64, notif_id: i64) -> AppResult<()> {
        let notification = self
            .notification_repo
            .find_by_id(notif_id)
            .await
            .map_err(|_| AppError::BadRequest("notification not found".to_string()))?;

        if notification.recipient_type == "user"
            && notification.user_id.map(|v| v != user_id).unwrap_or(false)
        {
            return Err(AppError::BadRequest("unauthorized".to_string()));
        }

        self.notification_repo.mark_as_read(notif_id, user_id).await
    }

    pub async fn mark_all_as_read(&self, user_id: i64) -> AppResult<()> {
        self.notification_repo.mark_all_as_read(user_id).await
    }

    pub async fn delete_notification(&self, user_id: i64, notif_id: i64) -> AppResult<()> {
        let notification = self
            .notification_repo
            .find_by_id(notif_id)
            .await
            .map_err(|_| AppError::BadRequest("notification not found".to_string()))?;

        if notification.user_id.map(|v| v != user_id).unwrap_or(true) {
            return Err(AppError::BadRequest("can only delete personal notifications".to_string()));
        }

        self.notification_repo.delete(notif_id).await
    }

    pub async fn delete_all_notifications(&self, user_id: i64) -> AppResult<()> {
        self.notification_repo.delete_all_by_user_id(user_id).await
    }
}
