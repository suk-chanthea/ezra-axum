//! Favorite use case, mirroring Go `favoriteUseCase`.

use std::sync::Arc;

use crate::domain::entity::Favorite;
use crate::domain::repository::{FavoriteRepository, MusicRepository};
use crate::error::{AppError, AppResult};
use crate::interface::http::dto::response::{MusicResponse, PaginationMetadata};

pub struct FavoriteUseCase {
    favorite_repo: Arc<dyn FavoriteRepository>,
    music_repo: Arc<dyn MusicRepository>,
}

impl FavoriteUseCase {
    pub fn new(
        favorite_repo: Arc<dyn FavoriteRepository>,
        music_repo: Arc<dyn MusicRepository>,
    ) -> Self {
        FavoriteUseCase { favorite_repo, music_repo }
    }

    pub async fn add_favorite(&self, user_id: i64, music_id: i64) -> AppResult<()> {
        self.music_repo
            .find_by_id(music_id)
            .await
            .map_err(|_| AppError::BadRequest("music not found".to_string()))?;

        if self.favorite_repo.is_favorite(user_id, music_id).await? {
            return Err(AppError::BadRequest("already favorited".to_string()));
        }

        let mut favorite = Favorite::new(user_id, music_id);
        if !favorite.is_valid() {
            return Err(AppError::BadRequest("invalid favorite data".to_string()));
        }
        self.favorite_repo.create(&mut favorite).await
    }

    pub async fn remove_favorite(&self, user_id: i64, music_id: i64) -> AppResult<()> {
        if !self.favorite_repo.is_favorite(user_id, music_id).await? {
            return Err(AppError::BadRequest("favorite not found".to_string()));
        }
        self.favorite_repo.delete(user_id, music_id).await
    }

    pub async fn get_user_favorites(&self, user_id: i64) -> AppResult<Vec<MusicResponse>> {
        let musics = self.favorite_repo.get_by_user_id(user_id).await?;
        Ok(MusicResponse::list(&musics))
    }

    pub async fn get_user_favorites_paginated(
        &self,
        user_id: i64,
        page: i64,
        page_size: i64,
    ) -> AppResult<(Vec<MusicResponse>, PaginationMetadata)> {
        let offset = (page - 1) * page_size;
        let (musics, total) = self
            .favorite_repo
            .get_by_user_id_paginated(user_id, offset, page_size)
            .await?;
        Ok((MusicResponse::list(&musics), PaginationMetadata::new(page, page_size, total)))
    }

    pub async fn is_favorite(&self, user_id: i64, music_id: i64) -> AppResult<bool> {
        self.favorite_repo.is_favorite(user_id, music_id).await
    }

    pub async fn get_favorite_count(&self, music_id: i64) -> AppResult<i64> {
        self.favorite_repo.get_favorite_count(music_id).await
    }
}
