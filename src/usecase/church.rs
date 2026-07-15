//! Church use case, mirroring Go `churchUseCase`.

use std::sync::Arc;

use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};

use crate::domain::entity::{self, Church};
use crate::domain::repository::{ChurchRepository, UserRepository};
use crate::error::{AppError, AppResult};
use crate::interface::http::dto::request::CreateChurchRequest;
use crate::interface::http::dto::response::{ChurchResponse, PaginatedResponse, UserResponse};

pub struct ChurchUseCase {
    church_repo: Arc<dyn ChurchRepository>,
    user_repo: Arc<dyn UserRepository>,
}

fn clamp_page(page: i64, page_size: i64) -> (i64, i64) {
    let page = if page < 1 { 1 } else { page };
    let page_size = if page_size < 1 {
        20
    } else if page_size > 100 {
        100
    } else {
        page_size
    };
    (page, page_size)
}

fn parse_established(value: &str) -> Option<chrono::DateTime<Utc>> {
    if value.is_empty() {
        return None;
    }
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .ok()
        .map(|d| Utc.from_utc_datetime(&d.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())))
}

impl ChurchUseCase {
    pub fn new(
        church_repo: Arc<dyn ChurchRepository>,
        user_repo: Arc<dyn UserRepository>,
    ) -> Self {
        ChurchUseCase { church_repo, user_repo }
    }

    async fn to_response(&self, church: &Church) -> ChurchResponse {
        let mut response = ChurchResponse::from_entity(church);
        if let Ok(approved) = self.church_repo.count_members(church.id, "approved").await {
            response.member_count = approved as i32;
        }
        if let Ok(pending) = self.church_repo.count_members(church.id, "pending").await {
            response.pending_count = pending as i32;
        }
        response
    }

    pub async fn create_church(
        &self,
        req: CreateChurchRequest,
        owner_id: i64,
    ) -> AppResult<ChurchResponse> {
        if self.church_repo.find_by_name(&req.name).await.is_ok() {
            return Err(AppError::BadRequest("church with this name already exists".to_string()));
        }

        let mut church = Church {
            name: req.name,
            address: req.address,
            phone: req.phone,
            email: req.email,
            website: req.website,
            pastor_name: req.pastor_name,
            description: req.description,
            logo: req.logo,
            denomination: req.denomination,
            owner_id: Some(owner_id),
            established_date: parse_established(&req.established_date),
            ..Default::default()
        };

        self.church_repo
            .create(&mut church)
            .await
            .map_err(|e| AppError::Internal(format!("failed to create church: {e}")))?;

        Ok(self.to_response(&church).await)
    }

    pub async fn get_church_by_id(&self, id: i64) -> AppResult<ChurchResponse> {
        let church = self
            .church_repo
            .find_by_id(id)
            .await
            .map_err(|_| AppError::NotFound("church not found".to_string()))?;
        Ok(self.to_response(&church).await)
    }

    pub async fn get_all_churches(
        &self,
        page: i64,
        page_size: i64,
    ) -> AppResult<PaginatedResponse<ChurchResponse>> {
        let (page, page_size) = clamp_page(page, page_size);
        let offset = (page - 1) * page_size;
        let churches = self.church_repo.find_all(page_size, offset).await?;
        let total = self.church_repo.count().await?;
        let mut data = Vec::with_capacity(churches.len());
        for c in &churches {
            data.push(self.to_response(c).await);
        }
        Ok(PaginatedResponse::new(data, page, page_size, total))
    }

    pub async fn get_churches_by_denomination(
        &self,
        denomination: &str,
        page: i64,
        page_size: i64,
    ) -> AppResult<PaginatedResponse<ChurchResponse>> {
        let (page, page_size) = clamp_page(page, page_size);
        let offset = (page - 1) * page_size;
        let churches = self.church_repo.find_by_denomination(denomination, page_size, offset).await?;
        let total = churches.len() as i64;
        let mut data = Vec::with_capacity(churches.len());
        for c in &churches {
            data.push(self.to_response(c).await);
        }
        Ok(PaginatedResponse::new(data, page, page_size, total))
    }

    pub async fn update_church(
        &self,
        id: i64,
        req: CreateChurchRequest,
        user_id: i64,
    ) -> AppResult<ChurchResponse> {
        let mut church = self
            .church_repo
            .find_by_id(id)
            .await
            .map_err(|_| AppError::NotFound("church not found".to_string()))?;

        if church.owner_id.map(|v| v != user_id).unwrap_or(true) {
            return Err(AppError::BadRequest("only church owner can update church details".to_string()));
        }

        if req.name != church.name {
            if let Ok(existing) = self.church_repo.find_by_name(&req.name).await {
                if existing.id != id {
                    return Err(AppError::BadRequest("church with this name already exists".to_string()));
                }
            }
        }

        church.name = req.name;
        church.address = req.address;
        church.phone = req.phone;
        church.email = req.email;
        church.website = req.website;
        church.pastor_name = req.pastor_name;
        church.description = req.description;
        church.logo = req.logo;
        church.denomination = req.denomination;
        church.established_date = parse_established(&req.established_date);

        self.church_repo
            .update(&church)
            .await
            .map_err(|e| AppError::Internal(format!("failed to update church: {e}")))?;

        Ok(self.to_response(&church).await)
    }

    pub async fn delete_church(&self, id: i64, user_id: i64) -> AppResult<()> {
        let church = self
            .church_repo
            .find_by_id(id)
            .await
            .map_err(|_| AppError::NotFound("church not found".to_string()))?;
        if church.owner_id.map(|v| v != user_id).unwrap_or(true) {
            return Err(AppError::BadRequest("only church owner can delete the church".to_string()));
        }
        self.church_repo.delete(id).await
    }

    pub async fn join_church(&self, user_id: i64, church_id: i64) -> AppResult<()> {
        self.church_repo
            .find_by_id(church_id)
            .await
            .map_err(|_| AppError::BadRequest("church not found".to_string()))?;

        let mut user = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(|_| AppError::BadRequest("user not found".to_string()))?;

        if user.church_id.is_some() {
            return Err(AppError::BadRequest(
                "user is already a member of a church. Please leave current church first".to_string(),
            ));
        }

        user.church_id = Some(church_id);
        user.church_status = entity::church_status::PENDING.to_string();
        self.user_repo.update(&user).await
    }

    pub async fn leave_church(&self, user_id: i64) -> AppResult<()> {
        let mut user = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(|_| AppError::BadRequest("user not found".to_string()))?;

        if user.church_id.is_none() {
            return Err(AppError::BadRequest("user is not a member of any church".to_string()));
        }

        user.church_id = None;
        user.church_status = entity::church_status::PENDING.to_string();
        self.user_repo.update(&user).await
    }

    pub async fn approve_member(
        &self,
        church_id: i64,
        owner_id: i64,
        target_user_id: i64,
        status: &str,
    ) -> AppResult<()> {
        let church = self
            .church_repo
            .find_by_id(church_id)
            .await
            .map_err(|_| AppError::BadRequest("church not found".to_string()))?;

        if church.owner_id.map(|v| v != owner_id).unwrap_or(true) {
            return Err(AppError::BadRequest("only church owner can approve members".to_string()));
        }

        let mut user = self
            .user_repo
            .find_by_id(target_user_id)
            .await
            .map_err(|_| AppError::BadRequest("user not found".to_string()))?;

        if user.church_id.map(|v| v != church_id).unwrap_or(true) {
            return Err(AppError::BadRequest("user is not requesting to join this church".to_string()));
        }

        match status {
            "approved" => user.church_status = entity::church_status::APPROVED.to_string(),
            "rejected" => {
                user.church_status = entity::church_status::REJECTED.to_string();
                user.church_id = None;
            }
            _ => {
                return Err(AppError::BadRequest(
                    "invalid status. Must be 'approved' or 'rejected'".to_string(),
                ))
            }
        }

        self.user_repo.update(&user).await
    }

    pub async fn get_pending_members(
        &self,
        church_id: i64,
        owner_id: i64,
        page: i64,
        page_size: i64,
    ) -> AppResult<PaginatedResponse<UserResponse>> {
        let church = self
            .church_repo
            .find_by_id(church_id)
            .await
            .map_err(|_| AppError::BadRequest("church not found".to_string()))?;
        if church.owner_id.map(|v| v != owner_id).unwrap_or(true) {
            return Err(AppError::BadRequest("only church owner can view pending members".to_string()));
        }

        let (page, page_size) = clamp_page(page, page_size);
        let offset = (page - 1) * page_size;
        let users = self.church_repo.find_members(church_id, "pending", page_size, offset).await?;
        let total = self.church_repo.count_members(church_id, "pending").await?;
        Ok(PaginatedResponse::new(UserResponse::list(&users), page, page_size, total))
    }

    pub async fn get_approved_members(
        &self,
        church_id: i64,
        page: i64,
        page_size: i64,
    ) -> AppResult<PaginatedResponse<UserResponse>> {
        self.church_repo
            .find_by_id(church_id)
            .await
            .map_err(|_| AppError::BadRequest("church not found".to_string()))?;

        let (page, page_size) = clamp_page(page, page_size);
        let offset = (page - 1) * page_size;
        let users = self.church_repo.find_members(church_id, "approved", page_size, offset).await?;
        let total = self.church_repo.count_members(church_id, "approved").await?;
        Ok(PaginatedResponse::new(UserResponse::list(&users), page, page_size, total))
    }
}
