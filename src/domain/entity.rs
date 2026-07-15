//! Core business entities. These are framework-agnostic and mirror the Go `domain/entity` package.

use chrono::{DateTime, NaiveDate, Utc};

// ---- Constants mirroring the typed string constants in Go ----

pub mod church_status {
    pub const PENDING: &str = "pending";
    pub const APPROVED: &str = "approved";
    pub const REJECTED: &str = "rejected";
}

pub mod booking_status {
    pub const PENDING: &str = "pending";
    pub const CONFIRMED: &str = "confirmed";
    pub const CANCELLED: &str = "cancelled";
}

pub mod donation_type {
    pub const DONATE: &str = "donate";
    pub const SPONSOR: &str = "sponsor";
}

pub mod donor_type {
    pub const USER: &str = "user";
    pub const COMPANY: &str = "company";
    pub const ORGANIZATION: &str = "organization";
    pub const CHURCH: &str = "church";
}

pub mod donation_status {
    pub const PENDING: &str = "pending";
    pub const COMPLETED: &str = "completed";
    pub const FAILED: &str = "failed";
    pub const REFUNDED: &str = "refunded";
}

pub mod otp_purpose {
    pub const EMAIL_VERIFICATION: &str = "email_verification";
    pub const PASSWORD_RESET: &str = "password_reset";
    pub const LOGIN: &str = "login";
}

pub mod chat_message_type {
    pub const TEXT: &str = "text";
    pub const VOICE: &str = "voice";
    pub const IMAGE: &str = "image";
    pub const SONG: &str = "song";
}

pub mod conversation_type {
    pub const PRIVATE: &str = "private";
    pub const GROUP: &str = "group";
    pub const BAND: &str = "band";
}

pub mod chat_role {
    pub const OWNER: &str = "owner";
    pub const ADMIN: &str = "admin";
    pub const MEMBER: &str = "member";

    /// Whether `role` is allowed to manage members/messages (owner or admin).
    pub fn is_privileged(role: &str) -> bool {
        role == OWNER || role == ADMIN
    }
}

// ---- Entities ----

#[derive(Debug, Clone, Default)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub name: String,
    pub profile: String,
    pub email: String,
    pub email_verified: bool,
    pub phone: String,
    pub password: String,
    pub role: String,
    pub token: String,
    pub provider: String,
    pub provider_id: String,
    pub birthday: Option<NaiveDate>,
    pub church_id: Option<i64>,
    pub church_status: String,
    pub band_id: Option<i64>,
    pub bio: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// Creates a new user entity for local registration.
    pub fn new_local(username: String, name: String, email: String, password: String) -> Self {
        let now = Utc::now();
        User {
            username,
            name,
            email,
            email_verified: false,
            password,
            role: "user".to_string(),
            provider: "local".to_string(),
            church_status: church_status::PENDING.to_string(),
            created_at: now,
            updated_at: now,
            ..Default::default()
        }
    }

    /// Creates a new user entity for OAuth providers.
    pub fn new_oauth(email: String, name: String, provider: String, provider_id: String) -> Self {
        let now = Utc::now();
        User {
            username: email.clone(),
            name,
            email,
            email_verified: true,
            password: String::new(),
            role: "user".to_string(),
            provider,
            provider_id,
            church_status: church_status::PENDING.to_string(),
            created_at: now,
            updated_at: now,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Music {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub genre: String,
    pub duration: i32,
    pub bpm: i32,
    pub key: String,
    pub cover: String,
    pub lyrics: String,
    pub description: String,
    pub user_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Music {
    pub fn is_valid(&self) -> bool {
        !self.title.is_empty() && self.user_id > 0
    }
}

#[derive(Debug, Clone, Default)]
pub struct Event {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub cover: String,
    pub location: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub user_id: i64,
    pub musics: Vec<Music>,
    pub music_ids: Vec<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Event {
    pub fn is_valid(&self) -> bool {
        if self.title.is_empty() || self.cover.is_empty() || self.location.is_empty() {
            return false;
        }
        !self.end_time.lt(&self.start_time)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Booking {
    pub id: i64,
    pub event_id: i64,
    pub user_id: i64,
    pub status: String,
    pub notes: String,
    pub event: Option<Event>,
    pub user: Option<User>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Booking {
    pub fn new(event_id: i64, user_id: i64, notes: String) -> Self {
        let now = Utc::now();
        Booking {
            event_id,
            user_id,
            status: booking_status::PENDING.to_string(),
            notes,
            created_at: now,
            updated_at: now,
            ..Default::default()
        }
    }

    pub fn is_valid(&self) -> bool {
        self.event_id != 0 && self.user_id != 0
    }
}

#[derive(Debug, Clone, Default)]
pub struct Favorite {
    pub id: i64,
    pub user_id: i64,
    pub music_id: i64,
    pub created_at: DateTime<Utc>,
}

impl Favorite {
    pub fn new(user_id: i64, music_id: i64) -> Self {
        Favorite {
            user_id,
            music_id,
            created_at: Utc::now(),
            ..Default::default()
        }
    }

    pub fn is_valid(&self) -> bool {
        self.user_id > 0 && self.music_id > 0
    }
}

#[derive(Debug, Clone, Default)]
pub struct Band {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub cover: String,
    pub is_public: bool,
    pub user_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Band {
    pub fn new(name: String, description: String, cover: String, is_public: bool, user_id: i64) -> Self {
        let now = Utc::now();
        Band {
            name,
            description,
            cover,
            is_public,
            user_id,
            created_at: now,
            updated_at: now,
            ..Default::default()
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.user_id > 0
    }
}

/// A chat conversation: a private (1:1) thread, a named group, or a band's chat.
#[derive(Debug, Clone, Default)]
pub struct Conversation {
    pub id: i64,
    pub r#type: String,
    pub name: String,
    pub band_id: Option<i64>,
    pub owner_id: i64,
    pub members: Vec<ConversationMember>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Conversation {
    pub fn new(r#type: String, name: String, owner_id: i64, band_id: Option<i64>) -> Self {
        let now = Utc::now();
        Conversation {
            r#type,
            name,
            band_id,
            owner_id,
            members: Vec::new(),
            created_at: now,
            updated_at: now,
            ..Default::default()
        }
    }

    pub fn is_private(&self) -> bool {
        self.r#type == conversation_type::PRIVATE
    }
}

/// A participant in a conversation, with their role and send permission.
#[derive(Debug, Clone, Default)]
pub struct ConversationMember {
    pub id: i64,
    pub conversation_id: i64,
    pub user_id: i64,
    pub role: String,
    pub can_send: bool,
    pub user: Option<User>,
    pub created_at: DateTime<Utc>,
}

impl ConversationMember {
    pub fn new(conversation_id: i64, user_id: i64, role: String, can_send: bool) -> Self {
        ConversationMember {
            conversation_id,
            user_id,
            role,
            can_send,
            user: None,
            created_at: Utc::now(),
            ..Default::default()
        }
    }

    /// Whether this member is permitted to post messages.
    pub fn may_send(&self) -> bool {
        self.can_send || chat_role::is_privileged(&self.role)
    }

    pub fn is_privileged(&self) -> bool {
        chat_role::is_privileged(&self.role)
    }
}

/// A message posted in a conversation. Supports plain text, a voice clip, an image,
/// or sharing one or more songs (`musics`/`music_ids`).
#[derive(Debug, Clone, Default)]
pub struct ChatMessage {
    pub id: i64,
    pub conversation_id: i64,
    pub sender_id: i64,
    pub message_type: String,
    pub content: String,
    pub media_url: String,
    pub duration: i32,
    pub music_ids: Vec<i64>,
    pub musics: Vec<Music>,
    pub sender: Option<User>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ChatMessage {
    pub fn new(
        conversation_id: i64,
        sender_id: i64,
        message_type: String,
        content: String,
        media_url: String,
        duration: i32,
        music_ids: Vec<i64>,
    ) -> Self {
        let now = Utc::now();
        ChatMessage {
            conversation_id,
            sender_id,
            message_type,
            content,
            media_url,
            duration,
            music_ids,
            musics: Vec::new(),
            sender: None,
            created_at: now,
            updated_at: now,
            ..Default::default()
        }
    }

    pub fn is_valid(&self) -> bool {
        if self.conversation_id == 0 || self.sender_id == 0 {
            return false;
        }
        match self.message_type.as_str() {
            chat_message_type::TEXT => !self.content.is_empty(),
            chat_message_type::VOICE | chat_message_type::IMAGE => !self.media_url.is_empty(),
            chat_message_type::SONG => !self.music_ids.is_empty(),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Church {
    pub id: i64,
    pub name: String,
    pub address: String,
    pub phone: String,
    pub email: String,
    pub website: String,
    pub pastor_name: String,
    pub description: String,
    pub logo: String,
    pub established_date: Option<DateTime<Utc>>,
    pub denomination: String,
    pub owner_id: Option<i64>,
    pub owner: Option<User>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct Notification {
    pub id: i64,
    pub user_id: Option<i64>,
    pub sender_id: Option<i64>,
    pub band_id: Option<i64>,
    pub recipient_type: String,
    pub title: String,
    pub message: String,
    pub r#type: String,
    pub related_type: String,
    pub related_id: Option<i64>,
    pub is_read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Notification {
    pub fn new_user(user_id: i64, title: String, message: String, notif_type: String) -> Self {
        let now = Utc::now();
        Notification {
            user_id: Some(user_id),
            recipient_type: "user".to_string(),
            title,
            message,
            r#type: notif_type,
            is_read: false,
            created_at: now,
            updated_at: now,
            ..Default::default()
        }
    }

    pub fn new_band(band_id: i64, title: String, message: String, notif_type: String) -> Self {
        let now = Utc::now();
        Notification {
            band_id: Some(band_id),
            recipient_type: "band".to_string(),
            title,
            message,
            r#type: notif_type,
            is_read: false,
            created_at: now,
            updated_at: now,
            ..Default::default()
        }
    }

    pub fn new_broadcast(title: String, message: String, notif_type: String) -> Self {
        let now = Utc::now();
        Notification {
            recipient_type: "all".to_string(),
            title,
            message,
            r#type: notif_type,
            is_read: false,
            created_at: now,
            updated_at: now,
            ..Default::default()
        }
    }

    pub fn is_valid(&self) -> bool {
        let valid_types = ["info", "success", "warning", "error", "booking", "music", "event"];
        let valid_recipients = ["user", "band", "all"];

        if self.title.is_empty()
            || self.message.is_empty()
            || !valid_types.contains(&self.r#type.as_str())
            || !valid_recipients.contains(&self.recipient_type.as_str())
        {
            return false;
        }

        match self.recipient_type.as_str() {
            "user" => self.user_id.map(|v| v > 0).unwrap_or(false) && self.band_id.is_none(),
            "band" => self.band_id.map(|v| v > 0).unwrap_or(false) && self.user_id.is_none(),
            "all" => self.user_id.is_none() && self.band_id.is_none(),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Donation {
    pub id: i64,
    pub r#type: String,
    pub donor_type: String,
    pub user_id: Option<i64>,
    pub supporter_id: Option<i64>,
    pub company_name: String,
    pub company_email: String,
    pub company_phone: String,
    pub amount: f64,
    pub currency: String,
    pub message: String,
    pub status: String,
    pub transaction_id: String,
    pub payment_method: String,
    pub qr_expires_at: Option<DateTime<Utc>>,
    pub event_id: Option<i64>,
    pub user: Option<User>,
    pub supporter: Option<Supporter>,
    pub event: Option<Event>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Donation {
    pub fn new_user(r#type: String, user_id: i64, amount: f64, currency: String, message: String) -> Self {
        let now = Utc::now();
        Donation {
            r#type,
            donor_type: donor_type::USER.to_string(),
            user_id: Some(user_id),
            amount,
            currency,
            message,
            status: donation_status::PENDING.to_string(),
            created_at: now,
            updated_at: now,
            ..Default::default()
        }
    }

    pub fn new_company(
        r#type: String,
        company_name: String,
        company_email: String,
        company_phone: String,
        amount: f64,
        currency: String,
        message: String,
    ) -> Self {
        let now = Utc::now();
        Donation {
            r#type,
            donor_type: donor_type::COMPANY.to_string(),
            company_name,
            company_email,
            company_phone,
            amount,
            currency,
            message,
            status: donation_status::PENDING.to_string(),
            created_at: now,
            updated_at: now,
            ..Default::default()
        }
    }

    pub fn is_valid(&self) -> bool {
        if self.r#type != donation_type::DONATE && self.r#type != donation_type::SPONSOR {
            return false;
        }
        let valid_donor = [donor_type::USER, donor_type::COMPANY, donor_type::ORGANIZATION, donor_type::CHURCH];
        if !valid_donor.contains(&self.donor_type.as_str()) {
            return false;
        }
        if self.amount <= 0.0 {
            return false;
        }
        if self.currency.is_empty() {
            return false;
        }
        if self.donor_type == donor_type::USER && self.user_id.map(|v| v == 0).unwrap_or(true) {
            return false;
        }
        if self.donor_type == donor_type::COMPANY
            || self.donor_type == donor_type::ORGANIZATION
            || self.donor_type == donor_type::CHURCH
        {
            let has_supporter = self.supporter_id.map(|v| v > 0).unwrap_or(false);
            let has_inline = !self.company_name.is_empty() && !self.company_email.is_empty();
            if !has_supporter && !has_inline {
                return false;
            }
        }
        true
    }

    pub fn complete(&mut self, transaction_id: String, payment_method: String) {
        self.status = donation_status::COMPLETED.to_string();
        self.transaction_id = transaction_id;
        self.payment_method = payment_method;
        self.updated_at = Utc::now();
    }

    pub fn fail(&mut self) {
        self.status = donation_status::FAILED.to_string();
        self.updated_at = Utc::now();
    }

    pub fn refund(&mut self) {
        self.status = donation_status::REFUNDED.to_string();
        self.updated_at = Utc::now();
    }

    pub fn set_event(&mut self, event_id: i64) {
        self.event_id = Some(event_id);
        self.updated_at = Utc::now();
    }

    pub fn set_qr_expiration(&mut self) {
        self.qr_expires_at = Some(Utc::now() + chrono::Duration::minutes(3));
        self.updated_at = Utc::now();
    }

    /// Remaining seconds before the QR code expires (0 if expired or unset).
    pub fn qr_time_remaining_secs(&self) -> i64 {
        match self.qr_expires_at {
            None => 0,
            Some(exp) => {
                let remaining = (exp - Utc::now()).num_seconds();
                remaining.max(0)
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Supporter {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub r#type: String,
    pub website: String,
    pub address: String,
    pub logo: String,
    pub description: String,
    pub user_id: Option<i64>,
    pub user: Option<User>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct Setting {
    pub id: i64,
    pub user_id: i64,
    pub language: String,
    pub theme: String,
    pub notify_on_booking: bool,
    pub notify_on_music: bool,
    pub notify_on_event: bool,
    pub enable_push_notifications: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Setting {
    pub fn new(user_id: i64) -> Self {
        let now = Utc::now();
        Setting {
            user_id,
            language: "en".to_string(),
            theme: "light".to_string(),
            notify_on_booking: true,
            notify_on_music: false,
            notify_on_event: true,
            enable_push_notifications: true,
            created_at: now,
            updated_at: now,
            ..Default::default()
        }
    }

    pub fn is_valid(&self) -> bool {
        let valid_themes = ["light", "dark", "auto"];
        self.user_id > 0 && valid_themes.contains(&self.theme.as_str())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Otp {
    pub id: i64,
    pub email: String,
    pub code: String,
    pub purpose: String,
    pub expires_at: DateTime<Utc>,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Otp {
    pub fn new(email: String, code: String, purpose: String, expiry_minutes: i64) -> Self {
        let now = Utc::now();
        Otp {
            email,
            code,
            purpose,
            expires_at: now + chrono::Duration::minutes(expiry_minutes),
            verified: false,
            created_at: now,
            updated_at: now,
            ..Default::default()
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.verified
    }

    pub fn mark_verified(&mut self) {
        self.verified = true;
        self.updated_at = Utc::now();
    }
}

#[derive(Debug, Clone, Default)]
pub struct DeviceToken {
    pub id: i64,
    pub user_id: i64,
    pub token: String,
    pub platform: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DeviceToken {
    pub fn new(user_id: i64, token: String, platform: String) -> Self {
        DeviceToken {
            user_id,
            token,
            platform,
            is_active: true,
            ..Default::default()
        }
    }

    pub fn is_valid(&self) -> bool {
        if self.user_id == 0 || self.token.is_empty() {
            return false;
        }
        let valid_platforms = ["ios", "android", "web"];
        valid_platforms.contains(&self.platform.as_str())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Session {
    pub id: i64,
    pub user_id: i64,
    pub device_id: String,
    pub device_name: String,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
