//! Band use case, mirroring Go `bandUseCase`.

use std::sync::Arc;

use crate::domain::entity::Band;
use crate::domain::repository::{BandRepository, MusicRepository};
use crate::error::{AppError, AppResult};
use crate::interface::http::dto::request::{CreateBandRequest, MusicOrder};
use crate::interface::http::dto::response::{
    BandResponse, MusicResponse, PaginationMetadata, UserResponse,
};

pub struct BandUseCase {
    band_repo: Arc<dyn BandRepository>,
    music_repo: Arc<dyn MusicRepository>,
}

impl BandUseCase {
    pub fn new(
        band_repo: Arc<dyn BandRepository>,
        music_repo: Arc<dyn MusicRepository>,
    ) -> Self {
        BandUseCase { band_repo, music_repo }
    }

    async fn to_response(&self, band: &Band, include_details: bool) -> BandResponse {
        let mut response = BandResponse::from_entity(band);
        response.member_count = self.band_repo.get_band_member_count(band.id).await.unwrap_or(0);
        if include_details {
            let musics = self.band_repo.get_band_musics(band.id).await.unwrap_or_default();
            response.music_count = musics.len() as i32;
            response.musics = Some(MusicResponse::list(&musics));
        }
        response
    }

    async fn to_responses(&self, bands: &[Band]) -> Vec<BandResponse> {
        let mut out = Vec::with_capacity(bands.len());
        for band in bands {
            out.push(self.to_response(band, false).await);
        }
        out
    }

    pub async fn create_band(&self, req: CreateBandRequest, user_id: i64) -> AppResult<()> {
        let mut band = Band::new(req.name, req.description, req.cover, req.is_public, user_id);
        if !band.is_valid() {
            return Err(AppError::BadRequest("invalid band data".to_string()));
        }
        self.band_repo.save(&mut band).await
    }

    pub async fn get_all_bands(&self) -> AppResult<Vec<BandResponse>> {
        let bands = self.band_repo.find_all().await?;
        Ok(self.to_responses(&bands).await)
    }

    pub async fn get_all_bands_paginated(
        &self,
        page: i64,
        page_size: i64,
    ) -> AppResult<(Vec<BandResponse>, PaginationMetadata)> {
        let offset = (page - 1) * page_size;
        let (bands, total) = self.band_repo.find_all_paginated(offset, page_size).await?;
        Ok((self.to_responses(&bands).await, PaginationMetadata::new(page, page_size, total)))
    }

    pub async fn get_band_by_id(&self, id: i64) -> AppResult<BandResponse> {
        let band = self.band_repo.find_by_id(id).await?;
        Ok(self.to_response(&band, true).await)
    }

    pub async fn get_bands_by_user_id(&self, user_id: i64) -> AppResult<Vec<BandResponse>> {
        let bands = self.band_repo.find_by_user_id(user_id).await?;
        Ok(self.to_responses(&bands).await)
    }

    pub async fn get_bands_by_user_id_paginated(
        &self,
        user_id: i64,
        page: i64,
        page_size: i64,
    ) -> AppResult<(Vec<BandResponse>, PaginationMetadata)> {
        let offset = (page - 1) * page_size;
        let (bands, total) = self.band_repo.find_by_user_id_paginated(user_id, offset, page_size).await?;
        Ok((self.to_responses(&bands).await, PaginationMetadata::new(page, page_size, total)))
    }

    pub async fn get_public_bands(&self) -> AppResult<Vec<BandResponse>> {
        let bands = self.band_repo.find_public_bands().await?;
        Ok(self.to_responses(&bands).await)
    }

    pub async fn get_public_bands_paginated(
        &self,
        page: i64,
        page_size: i64,
    ) -> AppResult<(Vec<BandResponse>, PaginationMetadata)> {
        let offset = (page - 1) * page_size;
        let (bands, total) = self.band_repo.find_public_bands_paginated(offset, page_size).await?;
        Ok((self.to_responses(&bands).await, PaginationMetadata::new(page, page_size, total)))
    }

    pub async fn update_band(&self, id: i64, req: CreateBandRequest, user_id: i64) -> AppResult<()> {
        let mut band = self
            .band_repo
            .find_by_id(id)
            .await
            .map_err(|_| AppError::BadRequest("band not found".to_string()))?;
        if band.user_id != user_id {
            return Err(AppError::BadRequest("unauthorized to update this band".to_string()));
        }
        band.name = req.name;
        band.description = req.description;
        band.cover = req.cover;
        band.is_public = req.is_public;
        if !band.is_valid() {
            return Err(AppError::BadRequest("invalid band data".to_string()));
        }
        self.band_repo.update(&band).await
    }

    pub async fn delete_band(&self, id: i64, user_id: i64) -> AppResult<()> {
        let band = self
            .band_repo
            .find_by_id(id)
            .await
            .map_err(|_| AppError::BadRequest("band not found".to_string()))?;
        if band.user_id != user_id {
            return Err(AppError::BadRequest("unauthorized to delete this band".to_string()));
        }
        self.band_repo.delete(id).await
    }

    pub async fn add_musics_to_band(
        &self,
        band_id: i64,
        music_ids: Vec<i64>,
        user_id: i64,
    ) -> AppResult<()> {
        let band = self
            .band_repo
            .find_by_id(band_id)
            .await
            .map_err(|_| AppError::BadRequest("band not found".to_string()))?;
        if band.user_id != user_id {
            return Err(AppError::BadRequest("unauthorized to modify this band".to_string()));
        }
        if !music_ids.is_empty() {
            let musics = self
                .music_repo
                .find_by_ids(&music_ids)
                .await
                .map_err(|_| AppError::BadRequest("failed to validate music IDs".to_string()))?;
            if musics.len() != music_ids.len() {
                return Err(AppError::BadRequest("one or more music IDs do not exist".to_string()));
            }
        }
        self.band_repo.add_musics_to_band(band_id, &music_ids).await
    }

    pub async fn remove_music_from_band(
        &self,
        band_id: i64,
        music_id: i64,
        user_id: i64,
    ) -> AppResult<()> {
        let band = self
            .band_repo
            .find_by_id(band_id)
            .await
            .map_err(|_| AppError::BadRequest("band not found".to_string()))?;
        if band.user_id != user_id {
            return Err(AppError::BadRequest("unauthorized to modify this band".to_string()));
        }
        self.band_repo.remove_music_from_band(band_id, music_id).await
    }

    pub async fn get_band_musics(&self, band_id: i64) -> AppResult<Vec<MusicResponse>> {
        let musics = self.band_repo.get_band_musics(band_id).await?;
        Ok(MusicResponse::list(&musics))
    }

    pub async fn reorder_band_musics(
        &self,
        band_id: i64,
        music_orders: Vec<MusicOrder>,
        user_id: i64,
    ) -> AppResult<()> {
        let band = self
            .band_repo
            .find_by_id(band_id)
            .await
            .map_err(|_| AppError::BadRequest("band not found".to_string()))?;
        if band.user_id != user_id {
            return Err(AppError::BadRequest("unauthorized to modify this band".to_string()));
        }
        let orders: Vec<(i64, i32)> =
            music_orders.iter().map(|o| (o.music_id, o.display_order)).collect();
        self.band_repo.reorder_band_musics(band_id, &orders).await
    }

    pub async fn get_band_members(&self, band_id: i64) -> AppResult<Vec<UserResponse>> {
        let members = self.band_repo.get_band_members(band_id).await?;
        Ok(UserResponse::list(&members))
    }
}
