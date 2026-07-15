//! Outbound response payloads, mirroring the Go `domain/dto` response structs (including `omitempty`).

use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
use serde::Serialize;
use serde_json::Value;

use super::local_time::LocalTime;
use crate::domain::entity::{
    Band, Booking, ChatMessage, Church, Conversation, ConversationMember, Donation, Event, Music,
    Notification, Setting, Supporter, User, Session,
};

fn is_zero_i32(v: &i32) -> bool {
    *v == 0
}
fn is_zero_i64(v: &i64) -> bool {
    *v == 0
}

fn naive_date_to_local(d: NaiveDate) -> LocalTime {
    let dt = d.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    LocalTime::new(Utc.from_utc_datetime(&dt))
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub message: String,
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub id: i64,
    pub device_id: String,
    pub device_name: String,
    pub expires_at: LocalTime,
    pub created_at: LocalTime,
}

impl SessionResponse {
    pub fn new(session: &Session) -> Self {
        Self {
            id: session.id,
            device_id: session.device_id.clone(),
            device_name: session.device_name.clone(),
            expires_at: LocalTime::new(session.expires_at),
            created_at: LocalTime::new(session.created_at),
        }
    }

    pub fn list(sessions: &[Session]) -> Vec<Self> {
        sessions.iter().map(Self::new).collect()
    }
}
#[derive(Debug, Serialize)]
pub struct OtpResponse {
    pub message: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<LocalTime>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub error: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub errors: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl SuccessResponse {
    pub fn message(msg: &str) -> Self {
        SuccessResponse { message: msg.to_string(), data: None }
    }
    pub fn with_data(msg: &str, data: Value) -> Self {
        SuccessResponse { message: msg.to_string(), data: Some(data) }
    }
}

#[derive(Debug, Serialize)]
pub struct MusicResponse {
    pub id: i64,
    pub title: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub artist: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub album: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub genre: String,
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub duration: i32,
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub bpm: i32,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub key: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub cover: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub lyrics: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    pub user_id: i64,
    pub created_at: LocalTime,
    pub updated_at: LocalTime,
}

impl MusicResponse {
    pub fn from_entity(m: &Music) -> Self {
        MusicResponse {
            id: m.id,
            title: m.title.clone(),
            artist: m.artist.clone(),
            album: m.album.clone(),
            genre: m.genre.clone(),
            duration: m.duration,
            bpm: m.bpm,
            key: m.key.clone(),
            cover: m.cover.clone(),
            lyrics: m.lyrics.clone(),
            description: m.description.clone(),
            user_id: m.user_id,
            created_at: LocalTime::new(m.created_at),
            updated_at: LocalTime::new(m.updated_at),
        }
    }

    pub fn list(items: &[Music]) -> Vec<MusicResponse> {
        items.iter().map(MusicResponse::from_entity).collect()
    }
}

#[derive(Debug, Serialize)]
pub struct EventResponse {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub cover: String,
    pub location: String,
    pub start_time: LocalTime,
    pub end_time: LocalTime,
    pub user_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub musics: Option<Vec<MusicResponse>>,
    pub created_at: LocalTime,
    pub updated_at: LocalTime,
}

impl EventResponse {
    pub fn from_entity(e: &Event) -> Self {
        let musics = if e.musics.is_empty() {
            None
        } else {
            Some(MusicResponse::list(&e.musics))
        };
        EventResponse {
            id: e.id,
            title: e.title.clone(),
            content: e.content.clone(),
            cover: e.cover.clone(),
            location: e.location.clone(),
            start_time: LocalTime::new(e.start_time),
            end_time: LocalTime::new(e.end_time),
            user_id: e.user_id,
            musics,
            created_at: LocalTime::new(e.created_at),
            updated_at: LocalTime::new(e.updated_at),
        }
    }

    pub fn list(items: &[Event]) -> Vec<EventResponse> {
        items.iter().map(EventResponse::from_entity).collect()
    }
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub profile: String,
    pub email: String,
    pub email_verified: bool,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub phone: String,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birthday: Option<LocalTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub church_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub church: Option<Box<ChurchResponse>>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub church_status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub band_id: Option<i64>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub bio: String,
    pub created_at: LocalTime,
    pub updated_at: LocalTime,
}

impl UserResponse {
    pub fn from_entity(u: &User) -> Self {
        UserResponse {
            id: u.id,
            username: u.username.clone(),
            name: u.name.clone(),
            profile: u.profile.clone(),
            email: u.email.clone(),
            email_verified: u.email_verified,
            phone: u.phone.clone(),
            role: u.role.clone(),
            birthday: u.birthday.map(naive_date_to_local),
            church_id: u.church_id,
            church: None,
            church_status: u.church_status.clone(),
            band_id: u.band_id,
            bio: u.bio.clone(),
            created_at: LocalTime::new(u.created_at),
            updated_at: LocalTime::new(u.updated_at),
        }
    }

    pub fn list(items: &[User]) -> Vec<UserResponse> {
        items.iter().map(UserResponse::from_entity).collect()
    }
}

#[derive(Debug, Serialize)]
pub struct BookingResponse {
    pub id: i64,
    pub event_id: i64,
    pub user_id: i64,
    pub status: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub notes: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<EventResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserResponse>,
    pub created_at: LocalTime,
    pub updated_at: LocalTime,
}

impl BookingResponse {
    pub fn from_entity(b: &Booking) -> Self {
        BookingResponse {
            id: b.id,
            event_id: b.event_id,
            user_id: b.user_id,
            status: b.status.clone(),
            notes: b.notes.clone(),
            event: b.event.as_ref().map(EventResponse::from_entity),
            user: b.user.as_ref().map(UserResponse::from_entity),
            created_at: LocalTime::new(b.created_at),
            updated_at: LocalTime::new(b.updated_at),
        }
    }

    pub fn list(items: &[Booking]) -> Vec<BookingResponse> {
        items.iter().map(BookingResponse::from_entity).collect()
    }
}

#[derive(Debug, Serialize)]
pub struct BandResponse {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub cover: String,
    pub is_public: bool,
    pub user_id: i64,
    #[serde(skip_serializing_if = "is_zero_i64")]
    pub member_count: i64,
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub music_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub musics: Option<Vec<MusicResponse>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<UserResponse>>,
    pub created_at: LocalTime,
    pub updated_at: LocalTime,
}

impl BandResponse {
    pub fn from_entity(b: &Band) -> Self {
        BandResponse {
            id: b.id,
            name: b.name.clone(),
            description: b.description.clone(),
            cover: b.cover.clone(),
            is_public: b.is_public,
            user_id: b.user_id,
            member_count: 0,
            music_count: 0,
            musics: None,
            members: None,
            created_at: LocalTime::new(b.created_at),
            updated_at: LocalTime::new(b.updated_at),
        }
    }

    pub fn list(items: &[Band]) -> Vec<BandResponse> {
        items.iter().map(BandResponse::from_entity).collect()
    }
}

#[derive(Debug, Serialize)]
pub struct ConversationMemberResponse {
    pub id: i64,
    pub conversation_id: i64,
    pub user_id: i64,
    pub role: String,
    pub can_send: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserResponse>,
    pub created_at: LocalTime,
}

impl ConversationMemberResponse {
    pub fn from_entity(m: &ConversationMember) -> Self {
        ConversationMemberResponse {
            id: m.id,
            conversation_id: m.conversation_id,
            user_id: m.user_id,
            role: m.role.clone(),
            can_send: m.can_send,
            user: m.user.as_ref().map(UserResponse::from_entity),
            created_at: LocalTime::new(m.created_at),
        }
    }

    pub fn list(items: &[ConversationMember]) -> Vec<ConversationMemberResponse> {
        items.iter().map(ConversationMemberResponse::from_entity).collect()
    }
}

#[derive(Debug, Serialize)]
pub struct ConversationResponse {
    pub id: i64,
    pub r#type: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub band_id: Option<i64>,
    pub owner_id: i64,
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub member_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<ConversationMemberResponse>>,
    pub created_at: LocalTime,
    pub updated_at: LocalTime,
}

impl ConversationResponse {
    pub fn from_entity(c: &Conversation) -> Self {
        let members = if c.members.is_empty() {
            None
        } else {
            Some(ConversationMemberResponse::list(&c.members))
        };
        ConversationResponse {
            id: c.id,
            r#type: c.r#type.clone(),
            name: c.name.clone(),
            band_id: c.band_id,
            owner_id: c.owner_id,
            member_count: c.members.len() as i32,
            members,
            created_at: LocalTime::new(c.created_at),
            updated_at: LocalTime::new(c.updated_at),
        }
    }

    pub fn list(items: &[Conversation]) -> Vec<ConversationResponse> {
        items.iter().map(ConversationResponse::from_entity).collect()
    }
}

#[derive(Debug, Serialize)]
pub struct ChatMessageResponse {
    pub id: i64,
    pub conversation_id: i64,
    pub sender_id: i64,
    pub message_type: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub content: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub media_url: String,
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub duration: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub musics: Option<Vec<MusicResponse>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender: Option<UserResponse>,
    pub created_at: LocalTime,
    pub updated_at: LocalTime,
}

impl ChatMessageResponse {
    pub fn from_entity(m: &ChatMessage) -> Self {
        let musics = if m.musics.is_empty() {
            None
        } else {
            Some(MusicResponse::list(&m.musics))
        };
        ChatMessageResponse {
            id: m.id,
            conversation_id: m.conversation_id,
            sender_id: m.sender_id,
            message_type: m.message_type.clone(),
            content: m.content.clone(),
            media_url: m.media_url.clone(),
            duration: m.duration,
            musics,
            sender: m.sender.as_ref().map(UserResponse::from_entity),
            created_at: LocalTime::new(m.created_at),
            updated_at: LocalTime::new(m.updated_at),
        }
    }

    pub fn list(items: &[ChatMessage]) -> Vec<ChatMessageResponse> {
        items.iter().map(ChatMessageResponse::from_entity).collect()
    }
}

#[derive(Debug, Serialize)]
pub struct SettingResponse {
    pub id: i64,
    pub user_id: i64,
    pub language: String,
    pub theme: String,
    pub notify_on_booking: bool,
    pub notify_on_music: bool,
    pub notify_on_event: bool,
    pub enable_push_notifications: bool,
    pub created_at: LocalTime,
    pub updated_at: LocalTime,
}

impl SettingResponse {
    pub fn from_entity(s: &Setting) -> Self {
        SettingResponse {
            id: s.id,
            user_id: s.user_id,
            language: s.language.clone(),
            theme: s.theme.clone(),
            notify_on_booking: s.notify_on_booking,
            notify_on_music: s.notify_on_music,
            notify_on_event: s.notify_on_event,
            enable_push_notifications: s.enable_push_notifications,
            created_at: LocalTime::new(s.created_at),
            updated_at: LocalTime::new(s.updated_at),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct NotificationResponse {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub band_id: Option<i64>,
    pub recipient_type: String,
    pub title: String,
    pub message: String,
    pub r#type: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub related_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_id: Option<i64>,
    pub is_read: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_at: Option<LocalTime>,
    pub created_at: LocalTime,
    pub updated_at: LocalTime,
}

impl NotificationResponse {
    pub fn from_entity(n: &Notification) -> Self {
        NotificationResponse {
            id: n.id,
            user_id: n.user_id,
            sender_id: n.sender_id,
            band_id: n.band_id,
            recipient_type: n.recipient_type.clone(),
            title: n.title.clone(),
            message: n.message.clone(),
            r#type: n.r#type.clone(),
            related_type: n.related_type.clone(),
            related_id: n.related_id,
            is_read: n.is_read,
            read_at: n.read_at.map(LocalTime::new),
            created_at: LocalTime::new(n.created_at),
            updated_at: LocalTime::new(n.updated_at),
        }
    }

    pub fn list(items: &[Notification]) -> Vec<NotificationResponse> {
        items.iter().map(NotificationResponse::from_entity).collect()
    }
}

#[derive(Debug, Serialize)]
pub struct SupporterResponse {
    pub id: i64,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub phone: String,
    pub r#type: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub website: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub address: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub logo: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserResponse>,
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub total_donations: i32,
    #[serde(skip_serializing_if = "is_zero_f64")]
    pub total_amount: f64,
    pub created_at: LocalTime,
    pub updated_at: LocalTime,
}

fn is_zero_f64(v: &f64) -> bool {
    *v == 0.0
}

impl SupporterResponse {
    pub fn from_entity(s: &Supporter) -> Self {
        SupporterResponse {
            id: s.id,
            name: s.name.clone(),
            email: s.email.clone(),
            phone: s.phone.clone(),
            r#type: s.r#type.clone(),
            website: s.website.clone(),
            address: s.address.clone(),
            logo: s.logo.clone(),
            description: s.description.clone(),
            user_id: s.user_id,
            user: s.user.as_ref().map(UserResponse::from_entity),
            total_donations: 0,
            total_amount: 0.0,
            created_at: LocalTime::new(s.created_at),
            updated_at: LocalTime::new(s.updated_at),
        }
    }

    pub fn list(items: &[Supporter]) -> Vec<SupporterResponse> {
        items.iter().map(SupporterResponse::from_entity).collect()
    }
}

#[derive(Debug, Serialize)]
pub struct ChurchResponse {
    pub id: i64,
    pub name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub address: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub phone: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub email: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub website: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub pastor_name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub logo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub established_date: Option<LocalTime>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub denomination: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Box<UserResponse>>,
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub member_count: i32,
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub pending_count: i32,
    pub created_at: LocalTime,
    pub updated_at: LocalTime,
}

impl ChurchResponse {
    pub fn from_entity(c: &Church) -> Self {
        ChurchResponse {
            id: c.id,
            name: c.name.clone(),
            address: c.address.clone(),
            phone: c.phone.clone(),
            email: c.email.clone(),
            website: c.website.clone(),
            pastor_name: c.pastor_name.clone(),
            description: c.description.clone(),
            logo: c.logo.clone(),
            established_date: c.established_date.map(LocalTime::new),
            denomination: c.denomination.clone(),
            owner_id: c.owner_id,
            owner: c.owner.as_ref().map(|u| Box::new(UserResponse::from_entity(u))),
            member_count: 0,
            pending_count: 0,
            created_at: LocalTime::new(c.created_at),
            updated_at: LocalTime::new(c.updated_at),
        }
    }

    pub fn list(items: &[Church]) -> Vec<ChurchResponse> {
        items.iter().map(ChurchResponse::from_entity).collect()
    }
}

#[derive(Debug, Serialize)]
pub struct DonationResponse {
    pub id: i64,
    pub r#type: String,
    pub donor_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supporter_id: Option<i64>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub company_name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub company_email: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub company_phone: String,
    pub amount: f64,
    pub currency: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub message: String,
    pub status: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub transaction_id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub payment_method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supporter: Option<SupporterResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<EventResponse>,
    pub created_at: LocalTime,
    pub updated_at: LocalTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_info: Option<InitiatePaymentResponse>,
}

impl DonationResponse {
    pub fn from_entity(d: &Donation) -> Self {
        DonationResponse {
            id: d.id,
            r#type: d.r#type.clone(),
            donor_type: d.donor_type.clone(),
            user_id: d.user_id,
            supporter_id: d.supporter_id,
            company_name: d.company_name.clone(),
            company_email: d.company_email.clone(),
            company_phone: d.company_phone.clone(),
            amount: d.amount,
            currency: d.currency.clone(),
            message: d.message.clone(),
            status: d.status.clone(),
            transaction_id: d.transaction_id.clone(),
            payment_method: d.payment_method.clone(),
            event_id: d.event_id,
            user: d.user.as_ref().map(UserResponse::from_entity),
            supporter: d.supporter.as_ref().map(SupporterResponse::from_entity),
            event: d.event.as_ref().map(EventResponse::from_entity),
            created_at: LocalTime::new(d.created_at),
            updated_at: LocalTime::new(d.updated_at),
            payment_info: None,
        }
    }

    pub fn list(items: &[Donation]) -> Vec<DonationResponse> {
        items.iter().map(DonationResponse::from_entity).collect()
    }
}

#[derive(Debug, Serialize)]
pub struct DonationStatsResponse {
    pub total_amount: f64,
    pub total_donations: i64,
    pub total_sponsors: i64,
    pub donate_amount: f64,
    pub sponsor_amount: f64,
    pub user_donations: i64,
    pub company_donations: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct InitiatePaymentResponse {
    pub donation_id: i64,
    pub transaction_id: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub payment_url: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub qr_code: String,
    pub amount: String,
    pub currency: String,
    pub payment_method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<LocalTime>,
    #[serde(skip_serializing_if = "is_zero_i32")]
    pub expires_in_seconds: i32,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct PaginationMetadata {
    pub current_page: i64,
    pub page_size: i64,
    pub total_pages: i64,
    pub total_records: i64,
    pub has_next_page: bool,
    pub has_prev_page: bool,
}

impl PaginationMetadata {
    pub fn new(page: i64, page_size: i64, total_records: i64) -> Self {
        let mut total_pages = if page_size > 0 {
            (total_records + page_size - 1) / page_size
        } else {
            1
        };
        if total_pages < 1 {
            total_pages = 1;
        }
        PaginationMetadata {
            current_page: page,
            page_size,
            total_pages,
            total_records,
            has_next_page: page < total_pages,
            has_prev_page: page > 1,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub pagination: PaginationMetadata,
}

impl<T: Serialize> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, page: i64, page_size: i64, total: i64) -> Self {
        PaginatedResponse {
            data,
            pagination: PaginationMetadata::new(page, page_size, total),
        }
    }
}
