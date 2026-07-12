//! OTP use case (send/verify/resend), mirroring Go `otpUseCase`.

use std::sync::Arc;

use rand::Rng;
use serde_json::json;

use crate::domain::entity::Otp;
use crate::domain::repository::{OtpRepository, UserRepository};
use crate::error::{AppError, AppResult};
use crate::infrastructure::email::{validate_email, EmailService};
use crate::interface::http::dto::local_time::LocalTime;
use crate::interface::http::dto::request::{SendOtpRequest, VerifyOtpRequest};
use crate::interface::http::dto::response::{OtpResponse, SuccessResponse};

pub struct OtpUseCase {
    otp_repo: Arc<dyn OtpRepository>,
    user_repo: Arc<dyn UserRepository>,
    email_service: Arc<dyn EmailService>,
    otp_expiry: i64,
}

impl OtpUseCase {
    pub fn new(
        otp_repo: Arc<dyn OtpRepository>,
        user_repo: Arc<dyn UserRepository>,
        email_service: Arc<dyn EmailService>,
        otp_expiry: i64,
    ) -> Self {
        let otp_expiry = if otp_expiry <= 0 { 10 } else { otp_expiry };
        OtpUseCase { otp_repo, user_repo, email_service, otp_expiry }
    }

    fn generate_code() -> String {
        let n: u32 = rand::thread_rng().gen_range(0..900000) + 100000;
        format!("{:06}", n)
    }

    pub async fn send_otp(&self, req: SendOtpRequest) -> AppResult<OtpResponse> {
        if !validate_email(&req.email) {
            return Err(AppError::BadRequest("invalid email format".to_string()));
        }

        if req.purpose == "email_verification" {
            if self.user_repo.find_by_email(&req.email).await.is_ok() {
                return Err(AppError::BadRequest(
                    "email already registered, please login or reset password".to_string(),
                ));
            }
        }

        if req.purpose == "login" || req.purpose == "password_reset" {
            if self.user_repo.find_by_email(&req.email).await.is_err() {
                return Err(AppError::BadRequest("email not found".to_string()));
            }
        }

        let code = Self::generate_code();

        let _ = self.otp_repo.delete_by_email(&req.email).await;

        let mut otp = Otp::new(req.email.clone(), code.clone(), req.purpose.clone(), self.otp_expiry);
        self.otp_repo
            .save(&mut otp)
            .await
            .map_err(|_| AppError::BadRequest("failed to save OTP".to_string()))?;

        self.email_service
            .send_otp(&req.email, &code, &req.purpose)
            .await
            .map_err(|e| AppError::BadRequest(format!("failed to send email: {e}")))?;

        Ok(OtpResponse {
            message: "OTP sent successfully to your email".to_string(),
            email: req.email,
            expires_at: Some(LocalTime::new(otp.expires_at)),
        })
    }

    pub async fn verify_otp(&self, req: VerifyOtpRequest) -> AppResult<SuccessResponse> {
        let mut otp = self
            .otp_repo
            .find_by_email_code_and_purpose(&req.email, &req.code, &req.purpose)
            .await
            .map_err(|_| AppError::BadRequest("invalid or expired OTP".to_string()))?;

        if otp.verified {
            return Err(AppError::BadRequest("OTP already used".to_string()));
        }
        if otp.is_expired() {
            return Err(AppError::BadRequest("OTP has expired".to_string()));
        }
        if !otp.is_valid() {
            return Err(AppError::BadRequest("invalid OTP".to_string()));
        }

        otp.mark_verified();
        self.otp_repo
            .update(&otp)
            .await
            .map_err(|_| AppError::BadRequest("failed to verify OTP".to_string()))?;

        Ok(SuccessResponse::with_data(
            "OTP verified successfully",
            json!({ "email": req.email, "purpose": req.purpose }),
        ))
    }

    #[allow(dead_code)]
    pub async fn resend_otp(&self, email: String, purpose: String) -> AppResult<OtpResponse> {
        self.send_otp(SendOtpRequest { email, purpose }).await
    }
}
