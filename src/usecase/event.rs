//! Event use case, mirroring Go `eventUseCase`.

use std::sync::Arc;

use crate::domain::entity::{Event, Notification};
use crate::domain::repository::{EventRepository, MusicRepository, NotificationRepository};
use crate::error::{AppError, AppResult};
use crate::interface::http::dto::request::CreateEventRequest;
use crate::interface::http::dto::response::{EventResponse, PaginationMetadata};

pub struct EventUseCase {
    event_repo: Arc<dyn EventRepository>,
    music_repo: Arc<dyn MusicRepository>,
    notification_repo: Arc<dyn NotificationRepository>,
}

impl EventUseCase {
    pub fn new(
        event_repo: Arc<dyn EventRepository>,
        music_repo: Arc<dyn MusicRepository>,
        notification_repo: Arc<dyn NotificationRepository>,
    ) -> Self {
        EventUseCase { event_repo, music_repo, notification_repo }
    }

    async fn validate_music_ids(&self, ids: &[i64]) -> AppResult<()> {
        if ids.is_empty() {
            return Ok(());
        }
        let musics = self
            .music_repo
            .find_by_ids(ids)
            .await
            .map_err(|_| AppError::BadRequest("failed to validate music IDs".to_string()))?;
        if musics.len() != ids.len() {
            return Err(AppError::BadRequest("one or more music IDs do not exist".to_string()));
        }
        Ok(())
    }

    pub async fn create_event(&self, req: CreateEventRequest, user_id: i64) -> AppResult<()> {
        self.validate_music_ids(&req.music_ids).await?;

        let mut event = Event {
            title: req.title,
            content: req.content,
            cover: req.cover,
            location: req.location,
            start_time: req.start_time,
            end_time: req.end_time,
            user_id,
            music_ids: req.music_ids,
            ..Default::default()
        };

        if !event.is_valid() {
            return Err(AppError::BadRequest("invalid event data".to_string()));
        }

        self.event_repo.save(&mut event).await?;

        let mut notification = Notification::new_broadcast(
            format!("New Event: {}", event.title),
            format!(
                "{} at {} on {}",
                event.title,
                event.location,
                event.start_time.format("%b %d, %Y %-I:%M %p")
            ),
            "event".to_string(),
        );
        notification.sender_id = Some(user_id);
        notification.related_type = "event".to_string();
        notification.related_id = Some(event.id);
        let _ = self.notification_repo.create(&mut notification).await;

        Ok(())
    }

    pub async fn get_all_events(&self) -> AppResult<Vec<EventResponse>> {
        let events = self.event_repo.find_all().await?;
        Ok(EventResponse::list(&events))
    }

    pub async fn get_all_events_paginated(
        &self,
        page: i64,
        page_size: i64,
    ) -> AppResult<(Vec<EventResponse>, PaginationMetadata)> {
        let offset = (page - 1) * page_size;
        let (events, total) = self.event_repo.find_all_paginated(offset, page_size).await?;
        Ok((EventResponse::list(&events), PaginationMetadata::new(page, page_size, total)))
    }

    pub async fn get_event_by_id(&self, id: i64) -> AppResult<EventResponse> {
        let event = self.event_repo.find_by_id(id).await?;
        Ok(EventResponse::from_entity(&event))
    }

    pub async fn get_events_by_user_id(&self, user_id: i64) -> AppResult<Vec<EventResponse>> {
        let events = self.event_repo.find_by_user_id(user_id).await?;
        Ok(EventResponse::list(&events))
    }

    pub async fn update_event(&self, id: i64, req: CreateEventRequest, user_id: i64) -> AppResult<()> {
        let mut existing = self
            .event_repo
            .find_by_id(id)
            .await
            .map_err(|_| AppError::BadRequest("event not found".to_string()))?;

        if existing.user_id != user_id {
            return Err(AppError::BadRequest("unauthorized to update this event".to_string()));
        }

        existing.title = req.title;
        existing.content = req.content;
        existing.cover = req.cover;
        existing.location = req.location;
        existing.start_time = req.start_time;
        existing.end_time = req.end_time;

        if !existing.is_valid() {
            return Err(AppError::BadRequest("invalid event data".to_string()));
        }

        self.event_repo.update(&existing).await?;

        // Update music associations (req.music_ids present == replace set).
        self.validate_music_ids(&req.music_ids).await?;
        let old_musics = self.event_repo.get_event_musics(id).await.unwrap_or_default();
        if !old_musics.is_empty() {
            let old_ids: Vec<i64> = old_musics.iter().map(|m| m.id).collect();
            let _ = self.event_repo.remove_musics_from_event(id, &old_ids).await;
        }
        if !req.music_ids.is_empty() {
            self.event_repo.add_musics_to_event(id, &req.music_ids).await?;
        }

        Ok(())
    }

    pub async fn delete_event(&self, id: i64, user_id: i64) -> AppResult<()> {
        let existing = self
            .event_repo
            .find_by_id(id)
            .await
            .map_err(|_| AppError::BadRequest("event not found".to_string()))?;
        if existing.user_id != user_id {
            return Err(AppError::BadRequest("unauthorized to delete this event".to_string()));
        }
        self.event_repo.delete(id).await
    }
}
