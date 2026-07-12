//! Supporter use case, mirroring Go `supporterUseCase`.

use std::sync::Arc;

use serde_json::{json, Value};

use crate::domain::entity::{self, Supporter};
use crate::domain::repository::{DonationRepository, SupporterRepository};
use crate::error::{AppError, AppResult};
use crate::interface::http::dto::request::{CreateSupporterRequest, UpdateSupporterRequest};
use crate::interface::http::dto::response::{PaginatedResponse, SupporterResponse};

pub struct SupporterUseCase {
    supporter_repo: Arc<dyn SupporterRepository>,
    donation_repo: Arc<dyn DonationRepository>,
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

impl SupporterUseCase {
    pub fn new(
        supporter_repo: Arc<dyn SupporterRepository>,
        donation_repo: Arc<dyn DonationRepository>,
    ) -> Self {
        SupporterUseCase { supporter_repo, donation_repo }
    }

    pub async fn create_supporter(
        &self,
        req: CreateSupporterRequest,
        user_id: Option<i64>,
    ) -> AppResult<SupporterResponse> {
        if self.supporter_repo.find_by_email(&req.email).await.is_ok() {
            return Err(AppError::BadRequest("supporter with this email already exists".to_string()));
        }

        let mut supporter = Supporter {
            name: req.name,
            email: req.email,
            phone: req.phone,
            r#type: req.r#type,
            website: req.website,
            address: req.address,
            logo: req.logo,
            description: req.description,
            user_id,
            ..Default::default()
        };

        self.supporter_repo
            .create(&mut supporter)
            .await
            .map_err(|e| AppError::Internal(format!("failed to create supporter: {e}")))?;

        Ok(SupporterResponse::from_entity(&supporter))
    }

    pub async fn get_supporter_by_id(&self, id: i64) -> AppResult<SupporterResponse> {
        let supporter = self
            .supporter_repo
            .find_by_id(id)
            .await
            .map_err(|_| AppError::NotFound("supporter not found".to_string()))?;
        let mut response = SupporterResponse::from_entity(&supporter);

        if let Ok((total_donations, total_amount)) = self.compute_stats(id).await {
            response.total_donations = total_donations as i32;
            response.total_amount = total_amount;
        }

        Ok(response)
    }

    pub async fn get_supporter_by_email(&self, email: &str) -> AppResult<SupporterResponse> {
        let supporter = self
            .supporter_repo
            .find_by_email(email)
            .await
            .map_err(|_| AppError::NotFound("supporter not found".to_string()))?;
        Ok(SupporterResponse::from_entity(&supporter))
    }

    pub async fn get_all_supporters(
        &self,
        page: i64,
        page_size: i64,
    ) -> AppResult<PaginatedResponse<SupporterResponse>> {
        let (page, page_size) = clamp_page(page, page_size);
        let offset = (page - 1) * page_size;
        let supporters = self.supporter_repo.find_all(page_size, offset).await?;
        let total = self.supporter_repo.count().await?;
        Ok(PaginatedResponse::new(
            SupporterResponse::list(&supporters),
            page,
            page_size,
            total,
        ))
    }

    pub async fn get_supporters_by_type(
        &self,
        supporter_type: &str,
        page: i64,
        page_size: i64,
    ) -> AppResult<PaginatedResponse<SupporterResponse>> {
        let (page, page_size) = clamp_page(page, page_size);
        let offset = (page - 1) * page_size;
        let supporters = self.supporter_repo.find_by_type(supporter_type, page_size, offset).await?;
        let total = self.supporter_repo.count_by_type(supporter_type).await?;
        Ok(PaginatedResponse::new(
            SupporterResponse::list(&supporters),
            page,
            page_size,
            total,
        ))
    }

    pub async fn get_supporters_by_user(
        &self,
        user_id: i64,
        page: i64,
        page_size: i64,
    ) -> AppResult<Vec<SupporterResponse>> {
        let page = if page < 1 { 1 } else { page };
        let page_size = if page_size < 1 { 20 } else { page_size };
        let offset = (page - 1) * page_size;
        let supporters = self.supporter_repo.find_by_user(user_id, page_size, offset).await?;
        Ok(SupporterResponse::list(&supporters))
    }

    pub async fn update_supporter(
        &self,
        id: i64,
        req: UpdateSupporterRequest,
        user_id: Option<i64>,
    ) -> AppResult<SupporterResponse> {
        let mut supporter = self
            .supporter_repo
            .find_by_id(id)
            .await
            .map_err(|_| AppError::NotFound("supporter not found".to_string()))?;

        if let (Some(uid), Some(owner)) = (user_id, supporter.user_id) {
            if owner != uid {
                return Err(AppError::BadRequest(
                    "you don't have permission to update this supporter".to_string(),
                ));
            }
        }

        if req.email != supporter.email {
            if let Ok(existing) = self.supporter_repo.find_by_email(&req.email).await {
                if existing.id != id {
                    return Err(AppError::BadRequest(
                        "supporter with this email already exists".to_string(),
                    ));
                }
            }
        }

        supporter.name = req.name;
        supporter.email = req.email;
        supporter.phone = req.phone;
        supporter.r#type = req.r#type;
        supporter.website = req.website;
        supporter.address = req.address;
        supporter.logo = req.logo;
        supporter.description = req.description;

        self.supporter_repo
            .update(&supporter)
            .await
            .map_err(|e| AppError::Internal(format!("failed to update supporter: {e}")))?;

        Ok(SupporterResponse::from_entity(&supporter))
    }

    pub async fn delete_supporter(&self, id: i64, user_id: Option<i64>) -> AppResult<()> {
        let supporter = self
            .supporter_repo
            .find_by_id(id)
            .await
            .map_err(|_| AppError::NotFound("supporter not found".to_string()))?;

        if let (Some(uid), Some(owner)) = (user_id, supporter.user_id) {
            if owner != uid {
                return Err(AppError::BadRequest(
                    "you don't have permission to delete this supporter".to_string(),
                ));
            }
        }

        self.supporter_repo.delete(id).await
    }

    async fn compute_stats(&self, supporter_id: i64) -> AppResult<(i64, f64)> {
        let donations = self.donation_repo.find_all(0, 0).await?;
        let mut total_donations = 0i64;
        let mut total_amount = 0.0;
        for d in &donations {
            if d.supporter_id == Some(supporter_id)
                && d.status == entity::donation_status::COMPLETED
            {
                total_donations += 1;
                total_amount += d.amount;
            }
        }
        Ok((total_donations, total_amount))
    }

    pub async fn get_supporter_stats(&self, supporter_id: i64) -> AppResult<Value> {
        let (total_donations, total_amount) = self.compute_stats(supporter_id).await?;
        Ok(json!({
            "total_donations": total_donations,
            "total_amount": total_amount,
        }))
    }
}
