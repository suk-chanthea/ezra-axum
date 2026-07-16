//! Inbound request payloads, mirroring the Go `domain/dto` request structs and their binding rules.

use serde::{Deserialize, Serialize};
use validator::Validate;

fn default_zero() -> i64 {
    0
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
    #[validate(length(min = 6, max = 6))]
    pub otp_code: String,
    #[serde(default)]
    pub profile: Option<String>,
    #[serde(default)]
    pub bio: Option<String>,
    #[serde(default)]
    pub birthday: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub device_id: Option<String>,
    #[serde(default)]
    pub device_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 1))]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
    #[serde(default)]
    pub otp_code: String,
    #[serde(default)]
    pub device_id: Option<String>,
    #[serde(default)]
    pub device_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct GoogleLoginRequest {
    #[validate(length(min = 1))]
    pub id_token: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SendOtpRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub purpose: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct VerifyOtpRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6, max = 6))]
    pub code: String,
    #[validate(length(min = 1))]
    pub purpose: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub new_password: String,
    #[validate(length(min = 6, max = 6))]
    pub otp_code: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(length(min = 3, max = 100))]
    pub username: String,
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[serde(default)]
    pub profile: String,
    #[serde(default)]
    pub phone: String,
    /// Format: YYYY-MM-DD
    #[serde(default)]
    pub birthday: String,
    #[serde(default)]
    pub bio: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChangeRoleRequest {
    #[validate(length(min = 1, max = 50))]
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateMusicRequest {
    pub title: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub album: String,
    #[serde(default)]
    pub genre: String,
    #[serde(default)]
    pub duration: i32,
    #[serde(default)]
    pub bpm: i32,
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub cover: String,
    #[serde(default)]
    pub lyrics: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateEventRequest {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    #[serde(default)]
    pub content: String,
    #[validate(length(min = 1, max = 200))]
    pub cover: String,
    #[validate(length(min = 1, max = 200))]
    pub location: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub music_ids: Vec<i64>,
}

pub type UpdateEventRequest = CreateEventRequest;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateBookingRequest {
    pub event_id: i64,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateBookingRequest {
    #[validate(length(min = 1))]
    pub status: String,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateBandRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub cover: String,
    #[serde(default)]
    pub is_public: bool,
}

pub type UpdateBandRequest = CreateBandRequest;

#[derive(Debug, Deserialize, Validate)]
pub struct AddMusicsRequest {
    #[validate(length(min = 1))]
    pub music_ids: Vec<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MusicOrder {
    pub music_id: i64,
    #[serde(default)]
    pub display_order: i32,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ReorderMusicsRequest {
    #[validate(length(min = 1))]
    pub music_orders: Vec<MusicOrder>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateConversationRequest {
    /// One of: `private`, `group`, `band`.
    #[validate(length(min = 1))]
    pub r#type: String,
    /// Display name (groups). Optional for private chats.
    #[serde(default)]
    pub name: String,
    /// Participants to add. For `private`, exactly one other user id.
    #[serde(default)]
    pub member_ids: Vec<i64>,
    /// Band to link when `type = band`.
    #[serde(default)]
    pub band_id: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddConversationMemberRequest {
    pub user_id: i64,
    /// Optional role: `admin` or `member` (defaults to `member`).
    #[serde(default)]
    pub role: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateConversationMemberRequest {
    /// New role: `admin` or `member` (empty = unchanged).
    #[serde(default)]
    pub role: String,
    /// New send permission (omitted = unchanged).
    #[serde(default)]
    pub can_send: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SendChatMessageRequest {
    /// One of: `text`, `voice`, `image`, `song`.
    #[validate(length(min = 1))]
    pub message_type: String,
    /// Text body (for `text`) or optional caption for other types.
    #[serde(default)]
    pub content: String,
    /// URL of the uploaded voice clip or image (for `voice`/`image`).
    #[serde(default)]
    pub media_url: String,
    /// Voice clip length in seconds (for `voice`).
    #[serde(default)]
    pub duration: i32,
    /// One or more song IDs to share (for `song`).
    #[serde(default)]
    pub music_ids: Vec<i64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSettingRequest {
    #[validate(length(min = 1))]
    pub language: String,
    #[validate(length(min = 1))]
    pub theme: String,
    #[serde(default)]
    pub notify_on_booking: bool,
    #[serde(default)]
    pub notify_on_music: bool,
    #[serde(default)]
    pub notify_on_event: bool,
    #[serde(default)]
    pub enable_push_notifications: bool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateNotificationRequest {
    #[serde(default)]
    pub user_id: Option<i64>,
    #[serde(default)]
    pub band_id: Option<i64>,
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[validate(length(min = 1))]
    pub message: String,
    #[validate(length(min = 1))]
    pub r#type: String,
    #[serde(default)]
    pub related_type: String,
    #[serde(default)]
    pub related_id: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateDonationRequest {
    #[validate(length(min = 1))]
    pub r#type: String,
    #[validate(length(min = 1))]
    pub donor_type: String,
    #[serde(default)]
    pub company_name: String,
    #[serde(default)]
    pub company_email: String,
    #[serde(default)]
    pub company_phone: String,
    pub amount: f64,
    #[validate(length(min = 1))]
    pub currency: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub event_id: Option<i64>,
    #[serde(default)]
    pub initiate_payment: bool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateDonationStatusRequest {
    #[validate(length(min = 1))]
    pub status: String,
    #[serde(default)]
    pub transaction_id: String,
    #[serde(default)]
    pub payment_method: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSupporterRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(email, length(max = 255))]
    pub email: String,
    #[serde(default)]
    pub phone: String,
    #[validate(length(min = 1))]
    pub r#type: String,
    #[serde(default)]
    pub website: String,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub logo: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub supporter_id: Option<i64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateSupporterRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(email, length(max = 255))]
    pub email: String,
    #[serde(default)]
    pub phone: String,
    #[validate(length(min = 1))]
    pub r#type: String,
    #[serde(default)]
    pub website: String,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub logo: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateChurchRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[serde(default)]
    pub address: String,
    #[serde(default)]
    pub phone: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub website: String,
    #[serde(default)]
    pub pastor_name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub logo: String,
    /// Format: YYYY-MM-DD
    #[serde(default)]
    pub established_date: String,
    #[serde(default)]
    pub denomination: String,
}

pub type UpdateChurchRequest = CreateChurchRequest;

#[derive(Debug, Deserialize, Validate)]
pub struct JoinChurchRequest {
    pub church_id: i64,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ApproveChurchMemberRequest {
    pub user_id: i64,
    #[validate(length(min = 1))]
    pub status: String,
}

// ---- Query parameters ----

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default)]
    pub page: i64,
    #[serde(default)]
    pub page_size: i64,
}

impl Default for PaginationQuery {
    fn default() -> Self {
        PaginationQuery { page: 0, page_size: 0 }
    }
}

impl PaginationQuery {
    pub fn get_page(&self) -> i64 {
        if self.page < 1 {
            1
        } else {
            self.page
        }
    }

    pub fn get_page_size(&self) -> i64 {
        if self.page_size < 1 {
            20
        } else if self.page_size > 100 {
            100
        } else {
            self.page_size
        }
    }

    pub fn get_offset(&self) -> i64 {
        (self.get_page() - 1) * self.get_page_size()
    }

    /// Whether the caller explicitly supplied pagination parameters.
    pub fn is_explicit(&self) -> bool {
        self.page > 0 || self.page_size > 0
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct DonationFilterQuery {
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub donor_type: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub event_id: Option<i64>,
    #[serde(default)]
    pub page: i64,
    #[serde(default)]
    pub page_size: i64,
}

impl DonationFilterQuery {
    pub fn get_page(&self) -> i64 {
        if self.page < 1 {
            1
        } else {
            self.page
        }
    }
    pub fn get_page_size(&self) -> i64 {
        if self.page_size < 1 {
            20
        } else if self.page_size > 100 {
            100
        } else {
            self.page_size
        }
    }
    pub fn get_offset(&self) -> i64 {
        (self.get_page() - 1) * self.get_page_size()
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct LimitOffsetQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default = "default_zero")]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

#[derive(Debug, Deserialize)]
pub struct EmailQuery {
    #[serde(default)]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AdminCreateUserRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
    #[validate(length(min = 1, max = 20))]
    pub role: String,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
}
