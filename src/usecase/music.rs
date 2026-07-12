//! Music use case, mirroring Go `musicUseCase`.

use std::sync::Arc;

use crate::domain::entity::Music;
use crate::domain::repository::MusicRepository;
use crate::error::{AppError, AppResult};
use crate::interface::http::dto::response::{MusicResponse, PaginationMetadata};

pub struct MusicUseCase {
    music_repo: Arc<dyn MusicRepository>,
}

impl MusicUseCase {
    pub fn new(music_repo: Arc<dyn MusicRepository>) -> Self {
        MusicUseCase { music_repo }
    }

    pub async fn create_music(&self, title: String, cover: String, user_id: i64) -> AppResult<()> {
        let mut music = Music { title, cover, user_id, ..Default::default() };
        if !music.is_valid() {
            return Err(AppError::BadRequest("invalid music data".to_string()));
        }
        self.music_repo.save(&mut music).await
    }

    pub async fn get_all_musics(&self) -> AppResult<Vec<MusicResponse>> {
        let musics = self.music_repo.find_all().await?;
        Ok(MusicResponse::list(&musics))
    }

    pub async fn get_all_musics_paginated(
        &self,
        page: i64,
        page_size: i64,
    ) -> AppResult<(Vec<MusicResponse>, PaginationMetadata)> {
        let offset = (page - 1) * page_size;
        let (musics, total) = self.music_repo.find_all_paginated(offset, page_size).await?;
        Ok((MusicResponse::list(&musics), PaginationMetadata::new(page, page_size, total)))
    }

    pub async fn get_music_by_id(&self, id: i64) -> AppResult<MusicResponse> {
        let music = self.music_repo.find_by_id(id).await?;
        Ok(MusicResponse::from_entity(&music))
    }

    pub async fn get_musics_by_user_id(&self, user_id: i64) -> AppResult<Vec<MusicResponse>> {
        let musics = self.music_repo.find_by_user_id(user_id).await?;
        Ok(MusicResponse::list(&musics))
    }

    pub async fn update_music(&self, id: i64, title: String, cover: String, user_id: i64) -> AppResult<()> {
        let mut music = self.music_repo.find_by_id(id).await?;
        if music.user_id != user_id {
            return Err(AppError::BadRequest("unauthorized".to_string()));
        }
        music.title = title;
        music.cover = cover;
        self.music_repo.update(&music).await
    }

    pub async fn delete_music(&self, id: i64, user_id: i64) -> AppResult<()> {
        let music = self.music_repo.find_by_id(id).await?;
        if music.user_id != user_id {
            return Err(AppError::BadRequest("unauthorized".to_string()));
        }
        self.music_repo.delete(id).await
    }
}
