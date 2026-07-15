//! Authentication use case (register, login, OAuth, profile), mirroring Go `authUseCase`.

use std::sync::Arc;

use chrono::NaiveDate;
use rand::Rng;

use crate::domain::entity::User;
use crate::domain::repository::{OtpRepository, UserRepository};
use crate::error::{AppError, AppResult};
use crate::infrastructure::security::jwt::{Claims, JwtManager};
use crate::infrastructure::security::password::{hash_password, verify_password};
use crate::interface::http::dto::request::{
    LoginRequest, RegisterRequest, ResetPasswordRequest, UpdateProfileRequest,
};
use crate::interface::http::dto::response::{AuthResponse, SuccessResponse, UserResponse};

pub struct AuthUseCase {
    user_repo: Arc<dyn UserRepository>,
    otp_repo: Arc<dyn OtpRepository>,
    jwt: JwtManager,
    #[allow(dead_code)]
    google_client_id: String,
    register_otp_required: bool,
    login_otp_required: bool,
}

impl AuthUseCase {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        otp_repo: Arc<dyn OtpRepository>,
        jwt: JwtManager,
        google_client_id: String,
        register_otp_required: bool,
        login_otp_required: bool,
    ) -> Self {
        AuthUseCase {
            user_repo,
            otp_repo,
            jwt,
            google_client_id,
            register_otp_required,
            login_otp_required,
        }
    }

    fn generate_token(&self, user: &User) -> AppResult<String> {
        self.jwt.generate(user.id, &user.username, &user.role)
    }

    pub async fn register(&self, req: RegisterRequest) -> AppResult<AuthResponse> {
        if self.register_otp_required {
            let otp = self
                .otp_repo
                .find_by_email_code_and_purpose(&req.email, &req.otp_code, "email_verification")
                .await
                .map_err(|_| AppError::BadRequest("invalid OTP code".to_string()))?;

            if !otp.verified {
                return Err(AppError::BadRequest(
                    "OTP not verified. Please verify OTP first via /otp/verify".to_string(),
                ));
            }
            if otp.is_expired() {
                return Err(AppError::BadRequest(
                    "OTP has expired. Please request a new one".to_string(),
                ));
            }
        }

        if self.user_repo.find_by_email(&req.email).await.is_ok() {
            return Err(AppError::BadRequest("email already exists".to_string()));
        }

        // Auto-generate a unique username instead of accepting one from the client.
        let mut username = generate_username(&req.email);
        while self.user_repo.find_by_username(&username).await.is_ok() {
            username = generate_username(&req.email);
        }

        let hash = hash_password(&req.password)?;
        let mut user = User::new_local(username, req.name, req.email.clone(), hash);
        user.email_verified = true;

        self.user_repo.save(&mut user).await?;

        let token = self.generate_token(&user)?;
        self.user_repo.update_token(user.id, &token).await?;

        let _ = self
            .otp_repo
            .delete_by_email_and_purpose(&req.email, "email_verification")
            .await;

        Ok(AuthResponse { message: "user registered successfully".to_string(), token })
    }

    pub async fn login(&self, req: LoginRequest) -> AppResult<AuthResponse> {
        let user = self
            .user_repo
            .find_by_username(&req.username)
            .await
            .map_err(|_| AppError::Unauthorized("user not found".to_string()))?;

        if !verify_password(&req.password, &user.password) {
            return Err(AppError::Unauthorized("invalid credentials".to_string()));
        }

        if self.login_otp_required || !req.otp_code.is_empty() {
            if self.login_otp_required && req.otp_code.is_empty() {
                return Err(AppError::Unauthorized(
                    "OTP code is required to login".to_string(),
                ));
            }
            let otp = self
                .otp_repo
                .find_by_email_code_and_purpose(&user.email, &req.otp_code, "login")
                .await
                .map_err(|_| AppError::Unauthorized("invalid 2FA OTP code".to_string()))?;
            if !otp.verified {
                return Err(AppError::Unauthorized(
                    "OTP not verified. Please verify OTP first via /otp/verify".to_string(),
                ));
            }
            if otp.is_expired() {
                return Err(AppError::Unauthorized(
                    "OTP has expired. Please request a new one".to_string(),
                ));
            }
            let _ = self.otp_repo.delete_by_email_and_purpose(&user.email, "login").await;
        }

        let token = self.generate_token(&user)?;
        self.user_repo.update_token(user.id, &token).await?;

        Ok(AuthResponse { message: String::new(), token })
    }

    pub async fn logout(&self, user_id: i64) -> AppResult<()> {
        self.user_repo.update_token(user_id, "").await
    }

    pub async fn delete_user(&self, user_id: i64) -> AppResult<()> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(|_| AppError::BadRequest("user not found".to_string()))?;
        self.user_repo
            .delete(user.id)
            .await
            .map_err(|_| AppError::BadRequest("failed to delete user".to_string()))
    }

    pub async fn google_login(
        &self,
        google_id: &str,
        email: &str,
        name: &str,
        profile_picture: &str,
    ) -> AppResult<AuthResponse> {
        let user = match self.user_repo.find_by_provider_id("google", google_id).await {
            Ok(mut existing) => {
                existing.name = name.to_string();
                existing.profile = profile_picture.to_string();
                existing.email = email.to_string();
                self.user_repo.update(&existing).await?;
                existing
            }
            Err(_) => {
                if let Ok(existing) = self.user_repo.find_by_email(email).await {
                    if existing.provider != "google" {
                        return Err(AppError::BadRequest(
                            "email already registered with different provider".to_string(),
                        ));
                    }
                }
                let mut new_user = User::new_oauth(
                    email.to_string(),
                    name.to_string(),
                    "google".to_string(),
                    google_id.to_string(),
                );
                new_user.profile = profile_picture.to_string();
                self.user_repo.save(&mut new_user).await?;
                new_user
            }
        };

        let token = self.generate_token(&user)?;
        self.user_repo.update_token(user.id, &token).await?;

        Ok(AuthResponse { message: "google login successful".to_string(), token })
    }

    pub async fn reset_password(&self, req: ResetPasswordRequest) -> AppResult<SuccessResponse> {
        let otp = self
            .otp_repo
            .find_by_email_code_and_purpose(&req.email, &req.otp_code, "password_reset")
            .await
            .map_err(|_| AppError::BadRequest("invalid OTP code".to_string()))?;

        if !otp.verified {
            return Err(AppError::BadRequest("OTP not verified. Please verify OTP first".to_string()));
        }
        if otp.is_expired() {
            return Err(AppError::BadRequest("OTP has expired. Please request a new one".to_string()));
        }

        let mut user = self
            .user_repo
            .find_by_email(&req.email)
            .await
            .map_err(|_| AppError::BadRequest("user not found".to_string()))?;

        user.password = hash_password(&req.new_password)
            .map_err(|_| AppError::BadRequest("failed to hash password".to_string()))?;

        self.user_repo
            .update(&user)
            .await
            .map_err(|_| AppError::BadRequest("failed to update password".to_string()))?;

        let _ = self.user_repo.update_token(user.id, "").await;
        let _ = self.otp_repo.delete_by_email_and_purpose(&req.email, "password_reset").await;

        Ok(SuccessResponse::message("password reset successfully"))
    }

    pub async fn get_me(&self, user_id: i64) -> AppResult<UserResponse> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(|_| AppError::NotFound("user not found".to_string()))?;
        Ok(UserResponse::from_entity(&user))
    }

    pub async fn update_me(&self, user_id: i64, req: UpdateProfileRequest) -> AppResult<UserResponse> {
        let mut user = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(|_| AppError::BadRequest("user not found".to_string()))?;

        if req.username != user.username {
            if let Ok(existing) = self.user_repo.find_by_username(&req.username).await {
                if existing.id != user_id {
                    return Err(AppError::BadRequest("username already taken".to_string()));
                }
            }
        }

        user.username = req.username;
        user.name = req.name;
        user.profile = req.profile;
        user.phone = req.phone;
        user.bio = req.bio;

        if !req.birthday.is_empty() {
            let birthday = NaiveDate::parse_from_str(&req.birthday, "%Y-%m-%d")
                .map_err(|_| AppError::BadRequest("invalid birthday format, use YYYY-MM-DD".to_string()))?;
            user.birthday = Some(birthday);
        } else {
            user.birthday = None;
        }

        self.user_repo
            .update(&user)
            .await
            .map_err(|_| AppError::BadRequest("failed to update profile".to_string()))?;

        Ok(UserResponse::from_entity(&user))
    }

    /// Validates a JWT and returns its claims (used by the auth middleware).
    pub fn validate_token(&self, token: &str) -> AppResult<Claims> {
        self.jwt.validate(token)
    }

    /// Ensures the presented token still matches what is stored for the user.
    pub async fn verify_token_in_database(&self, user_id: i64, token: &str) -> AppResult<()> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(|_| AppError::Unauthorized("user not found".to_string()))?;
        if user.token.is_empty() {
            return Err(AppError::Unauthorized("user has been logged out".to_string()));
        }
        if user.token != token {
            return Err(AppError::Unauthorized("token has been invalidated".to_string()));
        }
        Ok(())
    }
}

/// Derives a unique-ish username from the local part of an email address,
/// suffixed with a random number to avoid collisions.
fn generate_username(email: &str) -> String {
    let base: String = email
        .split('@')
        .next()
        .unwrap_or("user")
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();
    let base = if base.is_empty() { "user".to_string() } else { base };
    let suffix: u32 = rand::thread_rng().gen_range(0..100_000);
    format!("{base}{suffix}")
}
