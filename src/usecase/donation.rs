//! Donation use case, mirroring Go `donationUseCase`.

use std::sync::Arc;

use crate::domain::entity::{self, Donation};
use crate::domain::repository::{DonationRepository, EventRepository, UserRepository};
use crate::error::{AppError, AppResult};
use crate::infrastructure::payway::{format_amount, PaywayService};
use crate::interface::http::dto::local_time::LocalTime;
use crate::interface::http::dto::request::{
    CreateDonationRequest, DonationFilterQuery, UpdateDonationStatusRequest,
};
use crate::interface::http::dto::response::{
    DonationResponse, DonationStatsResponse, InitiatePaymentResponse, PaginationMetadata,
};

pub struct DonationUseCase {
    donation_repo: Arc<dyn DonationRepository>,
    user_repo: Arc<dyn UserRepository>,
    event_repo: Arc<dyn EventRepository>,
    payway_service: Arc<dyn PaywayService>,
}

impl DonationUseCase {
    pub fn new(
        donation_repo: Arc<dyn DonationRepository>,
        user_repo: Arc<dyn UserRepository>,
        event_repo: Arc<dyn EventRepository>,
        payway_service: Arc<dyn PaywayService>,
    ) -> Self {
        DonationUseCase { donation_repo, user_repo, event_repo, payway_service }
    }

    pub async fn create_donation(
        &self,
        req: CreateDonationRequest,
        user_id: Option<i64>,
    ) -> AppResult<DonationResponse> {
        let mut donation = if req.donor_type == "user" {
            let uid = user_id.filter(|v| *v != 0).ok_or_else(|| {
                AppError::BadRequest("user must be authenticated to make a donation".to_string())
            })?;
            self.user_repo
                .find_by_id(uid)
                .await
                .map_err(|_| AppError::BadRequest("user not found".to_string()))?;
            Donation::new_user(req.r#type.clone(), uid, req.amount, req.currency.clone(), req.message.clone())
        } else if matches!(req.donor_type.as_str(), "company" | "organization" | "church") {
            if req.company_name.is_empty() || req.company_email.is_empty() {
                return Err(AppError::BadRequest(
                    "company/organization/church name and email are required".to_string(),
                ));
            }
            let mut d = Donation::new_company(
                req.r#type.clone(),
                req.company_name.clone(),
                req.company_email.clone(),
                req.company_phone.clone(),
                req.amount,
                req.currency.clone(),
                req.message.clone(),
            );
            d.donor_type = req.donor_type.clone();
            d
        } else {
            return Err(AppError::BadRequest("invalid donor type".to_string()));
        };

        if let Some(event_id) = req.event_id {
            if event_id > 0 {
                self.event_repo
                    .find_by_id(event_id)
                    .await
                    .map_err(|_| AppError::BadRequest("event not found".to_string()))?;
                donation.set_event(event_id);
            }
        }

        if !donation.is_valid() {
            return Err(AppError::BadRequest("invalid donation data".to_string()));
        }

        self.donation_repo.save(&mut donation).await?;

        let created = self.donation_repo.find_by_id(donation.id).await?;
        let mut response = DonationResponse::from_entity(&created);

        if req.initiate_payment {
            match self.initiate_payment(created.id).await {
                Ok(payment_info) => response.payment_info = Some(payment_info),
                Err(e) => tracing::warn!("Failed to initiate payment: {e}"),
            }
        }

        Ok(response)
    }

    pub async fn initiate_payment(&self, donation_id: i64) -> AppResult<InitiatePaymentResponse> {
        let mut donation = self
            .donation_repo
            .find_by_id(donation_id)
            .await
            .map_err(|_| AppError::BadRequest("donation not found".to_string()))?;

        if donation.status == entity::donation_status::COMPLETED {
            return Err(AppError::BadRequest("donation already paid".to_string()));
        }

        let transaction_id = format!("DON-{}-{}", donation.id, donation.created_at.timestamp());

        let (customer_name, customer_email, customer_phone) = if donation.donor_type
            == entity::donor_type::USER
            && donation.user.is_some()
        {
            let u = donation.user.as_ref().unwrap();
            (u.fullname.clone(), u.email.clone(), String::new())
        } else {
            (donation.company_name.clone(), donation.company_email.clone(), donation.company_phone.clone())
        };

        let items = match &donation.event {
            Some(event) => format!("{} for {}", donation.r#type, event.title),
            None => format!("{} - Donation #{}", donation.r#type, donation.id),
        };

        let amount_str = format_amount(donation.amount, &donation.currency);

        let payment_resp = if donation.r#type == entity::donation_type::DONATE {
            let resp = self
                .payway_service
                .initiate_qr_payment(
                    &transaction_id,
                    &amount_str,
                    &donation.currency,
                    &customer_name,
                    &customer_email,
                    &customer_phone,
                    &items,
                )
                .await
                .map_err(|e| AppError::BadRequest(format!("failed to initiate payment: {e}")))?;
            donation.set_qr_expiration();
            resp
        } else if donation.r#type == entity::donation_type::SPONSOR {
            self.payway_service
                .initiate_card_payment(
                    &transaction_id,
                    &amount_str,
                    &donation.currency,
                    &customer_name,
                    &customer_email,
                    &customer_phone,
                    &items,
                )
                .await
                .map_err(|e| AppError::BadRequest(format!("failed to initiate payment: {e}")))?
        } else {
            return Err(AppError::BadRequest("invalid donation type".to_string()));
        };

        donation.transaction_id = transaction_id.clone();
        self.donation_repo
            .update(&donation)
            .await
            .map_err(|e| AppError::Internal(format!("failed to update donation: {e}")))?;

        let mut response = InitiatePaymentResponse {
            donation_id: donation.id,
            transaction_id,
            payment_url: String::new(),
            qr_code: String::new(),
            amount: amount_str,
            currency: donation.currency.clone(),
            payment_method: String::new(),
            expires_at: None,
            expires_in_seconds: 0,
            message: "Payment initiated successfully".to_string(),
        };

        if donation.r#type == entity::donation_type::DONATE {
            response.payment_method = "qr".to_string();
            response.qr_code = payment_resp.qr_code;
            response.expires_at = donation.qr_expires_at.map(LocalTime::new);
            if donation.qr_expires_at.is_some() {
                response.expires_in_seconds = donation.qr_time_remaining_secs() as i32;
            }
        } else {
            response.payment_method = "card".to_string();
            response.payment_url = payment_resp.payment_url;
        }

        Ok(response)
    }

    pub async fn handle_payment_callback(
        &self,
        transaction_id: &str,
        status: &str,
        approval_code: &str,
        payment_method: &str,
    ) -> AppResult<()> {
        let donations = self.donation_repo.find_all(-1, 0).await?;
        let mut donation = donations
            .into_iter()
            .find(|d| d.transaction_id == transaction_id)
            .ok_or_else(|| AppError::BadRequest("donation not found for transaction".to_string()))?;

        if status == "success" {
            donation.complete(approval_code.to_string(), payment_method.to_string());
        } else {
            donation.fail();
        }

        self.donation_repo.update(&donation).await
    }

    pub async fn get_all_donations(
        &self,
        filter: &DonationFilterQuery,
    ) -> AppResult<(Vec<DonationResponse>, PaginationMetadata)> {
        let limit = filter.get_page_size();
        let offset = filter.get_offset();

        let donations = if !filter.r#type.is_empty() {
            self.donation_repo.find_by_type(&filter.r#type, limit, offset).await?
        } else if !filter.donor_type.is_empty() {
            self.donation_repo.find_by_donor_type(&filter.donor_type, limit, offset).await?
        } else if !filter.status.is_empty() {
            self.donation_repo.find_by_status(&filter.status, limit, offset).await?
        } else if filter.event_id.map(|v| v > 0).unwrap_or(false) {
            self.donation_repo.find_by_event_id(filter.event_id.unwrap(), limit, offset).await?
        } else {
            self.donation_repo.find_all(limit, offset).await?
        };

        let total = self.donation_repo.count().await?;
        Ok((
            DonationResponse::list(&donations),
            PaginationMetadata::new(filter.get_page(), filter.get_page_size(), total),
        ))
    }

    pub async fn get_donation_by_id(&self, id: i64) -> AppResult<DonationResponse> {
        let donation = self.donation_repo.find_by_id(id).await?;
        Ok(DonationResponse::from_entity(&donation))
    }

    pub async fn get_donations_by_user_id(
        &self,
        user_id: i64,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<DonationResponse>> {
        let donations = self.donation_repo.find_by_user_id(user_id, limit, offset).await?;
        Ok(DonationResponse::list(&donations))
    }

    pub async fn get_donations_by_type(
        &self,
        donation_type: &str,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<DonationResponse>> {
        let donations = self.donation_repo.find_by_type(donation_type, limit, offset).await?;
        Ok(DonationResponse::list(&donations))
    }

    pub async fn get_donations_by_event_id(
        &self,
        event_id: i64,
        limit: i64,
        offset: i64,
    ) -> AppResult<Vec<DonationResponse>> {
        let donations = self.donation_repo.find_by_event_id(event_id, limit, offset).await?;
        Ok(DonationResponse::list(&donations))
    }

    pub async fn update_donation_status(
        &self,
        id: i64,
        req: UpdateDonationStatusRequest,
    ) -> AppResult<()> {
        let mut donation = self
            .donation_repo
            .find_by_id(id)
            .await
            .map_err(|_| AppError::BadRequest("donation not found".to_string()))?;

        match req.status.as_str() {
            s if s == entity::donation_status::COMPLETED => {
                donation.complete(req.transaction_id, req.payment_method)
            }
            s if s == entity::donation_status::FAILED => donation.fail(),
            s if s == entity::donation_status::REFUNDED => donation.refund(),
            other => donation.status = other.to_string(),
        }

        self.donation_repo.update(&donation).await
    }

    pub async fn delete_donation(&self, id: i64, user_id: i64) -> AppResult<()> {
        let donation = self
            .donation_repo
            .find_by_id(id)
            .await
            .map_err(|_| AppError::BadRequest("donation not found".to_string()))?;

        if donation.donor_type == entity::donor_type::USER
            && donation.user_id.map(|v| v != user_id).unwrap_or(true)
        {
            return Err(AppError::BadRequest("unauthorized to delete this donation".to_string()));
        }

        self.donation_repo.delete(id).await
    }

    pub async fn get_donation_stats(&self) -> AppResult<DonationStatsResponse> {
        let total_amount = self.donation_repo.get_total_amount().await?;
        let donate_amount = self
            .donation_repo
            .get_total_amount_by_type(entity::donation_type::DONATE)
            .await?;
        let sponsor_amount = self
            .donation_repo
            .get_total_amount_by_type(entity::donation_type::SPONSOR)
            .await?;
        let donate_count = self.donation_repo.count_by_type(entity::donation_type::DONATE).await?;
        let sponsor_count = self.donation_repo.count_by_type(entity::donation_type::SPONSOR).await?;
        let user_donations = self
            .donation_repo
            .find_by_donor_type(entity::donor_type::USER, -1, 0)
            .await?;
        let company_donations = self
            .donation_repo
            .find_by_donor_type(entity::donor_type::COMPANY, -1, 0)
            .await?;

        Ok(DonationStatsResponse {
            total_amount,
            total_donations: donate_count,
            total_sponsors: sponsor_count,
            donate_amount,
            sponsor_amount,
            user_donations: user_donations.len() as i64,
            company_donations: company_donations.len() as i64,
        })
    }

    pub async fn get_donation_stats_by_event_id(
        &self,
        event_id: i64,
    ) -> AppResult<DonationStatsResponse> {
        let total_amount = self.donation_repo.get_total_amount_by_event_id(event_id).await?;
        let donations = self.donation_repo.find_by_event_id(event_id, -1, 0).await?;

        let mut donate_amount = 0.0;
        let mut sponsor_amount = 0.0;
        let mut donate_count = 0i64;
        let mut sponsor_count = 0i64;
        let mut user_count = 0i64;
        let mut company_count = 0i64;

        for d in &donations {
            if d.r#type == entity::donation_type::DONATE {
                donate_amount += d.amount;
                donate_count += 1;
            } else if d.r#type == entity::donation_type::SPONSOR {
                sponsor_amount += d.amount;
                sponsor_count += 1;
            }
            if d.donor_type == entity::donor_type::USER {
                user_count += 1;
            } else if d.donor_type == entity::donor_type::COMPANY {
                company_count += 1;
            }
        }

        Ok(DonationStatsResponse {
            total_amount,
            total_donations: donate_count,
            total_sponsors: sponsor_count,
            donate_amount,
            sponsor_amount,
            user_donations: user_count,
            company_donations: company_count,
        })
    }
}
