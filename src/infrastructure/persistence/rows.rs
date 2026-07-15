//! sqlx row structs mapping database columns to domain entities.

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::FromRow;

use crate::domain::entity::{
    Band, Booking, ChatMessage, Church, Conversation, ConversationMember, DeviceToken, Donation,
    Event, Music, Notification, Otp, Setting, Supporter, User,
};

#[derive(FromRow)]
pub struct UserRow {
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

impl UserRow {
    pub fn into_entity(self) -> User {
        User {
            id: self.id,
            username: self.username,
            name: self.name,
            profile: self.profile,
            email: self.email,
            email_verified: self.email_verified,
            phone: self.phone,
            password: self.password,
            role: self.role,
            token: self.token,
            provider: self.provider,
            provider_id: self.provider_id,
            birthday: self.birthday,
            church_id: self.church_id,
            church_status: self.church_status,
            band_id: self.band_id,
            bio: self.bio,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(FromRow)]
pub struct MusicRow {
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

impl MusicRow {
    pub fn into_entity(self) -> Music {
        Music {
            id: self.id,
            title: self.title,
            artist: self.artist,
            album: self.album,
            genre: self.genre,
            duration: self.duration,
            bpm: self.bpm,
            key: self.key,
            cover: self.cover,
            lyrics: self.lyrics,
            description: self.description,
            user_id: self.user_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(FromRow)]
pub struct EventRow {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub cover: String,
    pub location: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub user_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl EventRow {
    pub fn into_entity(self) -> Event {
        Event {
            id: self.id,
            title: self.title,
            content: self.content,
            cover: self.cover,
            location: self.location,
            start_time: self.start_time,
            end_time: self.end_time,
            user_id: self.user_id,
            musics: Vec::new(),
            music_ids: Vec::new(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(FromRow)]
pub struct BookingRow {
    pub id: i64,
    pub event_id: i64,
    pub user_id: i64,
    pub status: String,
    pub notes: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BookingRow {
    pub fn into_entity(self) -> Booking {
        Booking {
            id: self.id,
            event_id: self.event_id,
            user_id: self.user_id,
            status: self.status,
            notes: self.notes,
            event: None,
            user: None,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(FromRow)]
pub struct BandRow {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub cover: String,
    pub is_public: bool,
    pub user_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BandRow {
    pub fn into_entity(self) -> Band {
        Band {
            id: self.id,
            name: self.name,
            description: self.description,
            cover: self.cover,
            is_public: self.is_public,
            user_id: self.user_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(FromRow)]
pub struct ConversationRow {
    pub id: i64,
    #[sqlx(rename = "type")]
    pub type_: String,
    pub name: String,
    pub band_id: Option<i64>,
    pub owner_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ConversationRow {
    pub fn into_entity(self) -> Conversation {
        Conversation {
            id: self.id,
            r#type: self.type_,
            name: self.name,
            band_id: self.band_id,
            owner_id: self.owner_id,
            members: Vec::new(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(FromRow)]
pub struct ConversationMemberRow {
    pub id: i64,
    pub conversation_id: i64,
    pub user_id: i64,
    pub role: String,
    pub can_send: bool,
    pub created_at: DateTime<Utc>,
}

impl ConversationMemberRow {
    pub fn into_entity(self) -> ConversationMember {
        ConversationMember {
            id: self.id,
            conversation_id: self.conversation_id,
            user_id: self.user_id,
            role: self.role,
            can_send: self.can_send,
            user: None,
            created_at: self.created_at,
        }
    }
}

#[derive(FromRow)]
pub struct ChatMessageRow {
    pub id: i64,
    pub conversation_id: i64,
    pub sender_id: i64,
    pub message_type: String,
    pub content: String,
    pub media_url: String,
    pub duration: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ChatMessageRow {
    pub fn into_entity(self) -> ChatMessage {
        ChatMessage {
            id: self.id,
            conversation_id: self.conversation_id,
            sender_id: self.sender_id,
            message_type: self.message_type,
            content: self.content,
            media_url: self.media_url,
            duration: self.duration,
            music_ids: Vec::new(),
            musics: Vec::new(),
            sender: None,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(FromRow)]
pub struct NotificationRow {
    pub id: i64,
    pub user_id: Option<i64>,
    pub sender_id: Option<i64>,
    pub band_id: Option<i64>,
    pub recipient_type: String,
    pub title: String,
    pub message: String,
    #[sqlx(rename = "type")]
    pub type_: String,
    pub related_type: String,
    pub related_id: Option<i64>,
    pub is_read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl NotificationRow {
    pub fn into_entity(self) -> Notification {
        Notification {
            id: self.id,
            user_id: self.user_id,
            sender_id: self.sender_id,
            band_id: self.band_id,
            recipient_type: self.recipient_type,
            title: self.title,
            message: self.message,
            r#type: self.type_,
            related_type: self.related_type,
            related_id: self.related_id,
            is_read: self.is_read,
            read_at: self.read_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(FromRow)]
pub struct DonationRow {
    pub id: i64,
    #[sqlx(rename = "type")]
    pub type_: String,
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DonationRow {
    pub fn into_entity(self) -> Donation {
        Donation {
            id: self.id,
            r#type: self.type_,
            donor_type: self.donor_type,
            user_id: self.user_id,
            supporter_id: self.supporter_id,
            company_name: self.company_name,
            company_email: self.company_email,
            company_phone: self.company_phone,
            amount: self.amount,
            currency: self.currency,
            message: self.message,
            status: self.status,
            transaction_id: self.transaction_id,
            payment_method: self.payment_method,
            qr_expires_at: self.qr_expires_at,
            event_id: self.event_id,
            user: None,
            supporter: None,
            event: None,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(FromRow)]
pub struct SupporterRow {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub phone: String,
    #[sqlx(rename = "type")]
    pub type_: String,
    pub website: String,
    pub address: String,
    pub logo: String,
    pub description: String,
    pub user_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl SupporterRow {
    pub fn into_entity(self) -> Supporter {
        Supporter {
            id: self.id,
            name: self.name,
            email: self.email,
            phone: self.phone,
            r#type: self.type_,
            website: self.website,
            address: self.address,
            logo: self.logo,
            description: self.description,
            user_id: self.user_id,
            user: None,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(FromRow)]
pub struct SettingRow {
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

impl SettingRow {
    pub fn into_entity(self) -> Setting {
        Setting {
            id: self.id,
            user_id: self.user_id,
            language: self.language,
            theme: self.theme,
            notify_on_booking: self.notify_on_booking,
            notify_on_music: self.notify_on_music,
            notify_on_event: self.notify_on_event,
            enable_push_notifications: self.enable_push_notifications,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(FromRow)]
pub struct OtpRow {
    pub id: i64,
    pub email: String,
    pub code: String,
    pub purpose: String,
    pub expires_at: DateTime<Utc>,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl OtpRow {
    pub fn into_entity(self) -> Otp {
        Otp {
            id: self.id,
            email: self.email,
            code: self.code,
            purpose: self.purpose,
            expires_at: self.expires_at,
            verified: self.verified,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(FromRow)]
pub struct ChurchRow {
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ChurchRow {
    pub fn into_entity(self) -> Church {
        Church {
            id: self.id,
            name: self.name,
            address: self.address,
            phone: self.phone,
            email: self.email,
            website: self.website,
            pastor_name: self.pastor_name,
            description: self.description,
            logo: self.logo,
            established_date: self.established_date,
            denomination: self.denomination,
            owner_id: self.owner_id,
            owner: None,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(FromRow)]
pub struct DeviceTokenRow {
    pub id: i64,
    pub user_id: i64,
    pub token: String,
    pub platform: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DeviceTokenRow {
    pub fn into_entity(self) -> DeviceToken {
        DeviceToken {
            id: self.id,
            user_id: self.user_id,
            token: self.token,
            platform: self.platform,
            is_active: self.is_active,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
