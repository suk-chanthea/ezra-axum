//! Repository contracts (ports). Implemented by the infrastructure layer.

use async_trait::async_trait;

use crate::domain::entity::{
    Band, Booking, ChatMessage, Church, Conversation, ConversationMember, DeviceToken, Donation,
    Event, Favorite, Music, Notification, Otp, Setting, Supporter, User, Session,
};
use crate::error::AppResult;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn save(&self, user: &mut User) -> AppResult<()>;
    async fn find_by_username(&self, username: &str) -> AppResult<User>;
    async fn find_by_email(&self, email: &str) -> AppResult<User>;
    async fn find_by_id(&self, id: i64) -> AppResult<User>;
    async fn find_by_provider_id(&self, provider: &str, provider_id: &str) -> AppResult<User>;
    async fn update(&self, user: &User) -> AppResult<()>;
    async fn update_token(&self, id: i64, token: &str) -> AppResult<()>;
    async fn delete(&self, id: i64) -> AppResult<()>;
}

#[async_trait]
pub trait MusicRepository: Send + Sync {
    async fn save(&self, music: &mut Music) -> AppResult<()>;
    async fn find_all(&self) -> AppResult<Vec<Music>>;
    async fn find_all_paginated(&self, offset: i64, limit: i64) -> AppResult<(Vec<Music>, i64)>;
    async fn find_by_id(&self, id: i64) -> AppResult<Music>;
    async fn find_by_ids(&self, ids: &[i64]) -> AppResult<Vec<Music>>;
    async fn find_by_user_id(&self, user_id: i64) -> AppResult<Vec<Music>>;
    async fn update(&self, music: &Music) -> AppResult<()>;
    async fn delete(&self, id: i64) -> AppResult<()>;
}

#[async_trait]
pub trait EventRepository: Send + Sync {
    async fn save(&self, event: &mut Event) -> AppResult<()>;
    async fn find_all(&self) -> AppResult<Vec<Event>>;
    async fn find_all_paginated(&self, offset: i64, limit: i64) -> AppResult<(Vec<Event>, i64)>;
    async fn find_by_id(&self, id: i64) -> AppResult<Event>;
    async fn find_by_user_id(&self, user_id: i64) -> AppResult<Vec<Event>>;
    async fn update(&self, event: &Event) -> AppResult<()>;
    async fn delete(&self, id: i64) -> AppResult<()>;
    async fn add_musics_to_event(&self, event_id: i64, music_ids: &[i64]) -> AppResult<()>;
    async fn remove_musics_from_event(&self, event_id: i64, music_ids: &[i64]) -> AppResult<()>;
    async fn get_event_musics(&self, event_id: i64) -> AppResult<Vec<Music>>;
}

#[async_trait]
pub trait BookingRepository: Send + Sync {
    async fn save(&self, booking: &mut Booking) -> AppResult<()>;
    async fn find_all(&self) -> AppResult<Vec<Booking>>;
    async fn find_all_paginated(&self, offset: i64, limit: i64) -> AppResult<(Vec<Booking>, i64)>;
    async fn find_by_id(&self, id: i64) -> AppResult<Booking>;
    async fn find_by_event_id(&self, event_id: i64) -> AppResult<Vec<Booking>>;
    async fn find_by_user_id(&self, user_id: i64) -> AppResult<Vec<Booking>>;
    async fn find_by_event_and_user(&self, event_id: i64, user_id: i64) -> AppResult<Booking>;
    async fn update(&self, booking: &Booking) -> AppResult<()>;
    async fn delete(&self, id: i64) -> AppResult<()>;
}

#[async_trait]
pub trait FavoriteRepository: Send + Sync {
    async fn create(&self, favorite: &mut Favorite) -> AppResult<()>;
    async fn delete(&self, user_id: i64, music_id: i64) -> AppResult<()>;
    async fn get_by_user_id(&self, user_id: i64) -> AppResult<Vec<Music>>;
    async fn get_by_user_id_paginated(
        &self,
        user_id: i64,
        offset: i64,
        limit: i64,
    ) -> AppResult<(Vec<Music>, i64)>;
    async fn is_favorite(&self, user_id: i64, music_id: i64) -> AppResult<bool>;
    async fn get_favorite_count(&self, music_id: i64) -> AppResult<i64>;
}

#[async_trait]
pub trait BandRepository: Send + Sync {
    async fn save(&self, band: &mut Band) -> AppResult<()>;
    async fn find_all(&self) -> AppResult<Vec<Band>>;
    async fn find_all_paginated(&self, offset: i64, limit: i64) -> AppResult<(Vec<Band>, i64)>;
    async fn find_by_id(&self, id: i64) -> AppResult<Band>;
    async fn find_by_user_id(&self, user_id: i64) -> AppResult<Vec<Band>>;
    async fn find_by_user_id_paginated(
        &self,
        user_id: i64,
        offset: i64,
        limit: i64,
    ) -> AppResult<(Vec<Band>, i64)>;
    async fn find_public_bands(&self) -> AppResult<Vec<Band>>;
    async fn find_public_bands_paginated(&self, offset: i64, limit: i64) -> AppResult<(Vec<Band>, i64)>;
    async fn update(&self, band: &Band) -> AppResult<()>;
    async fn delete(&self, id: i64) -> AppResult<()>;
    async fn add_musics_to_band(&self, band_id: i64, music_ids: &[i64]) -> AppResult<()>;
    async fn remove_music_from_band(&self, band_id: i64, music_id: i64) -> AppResult<()>;
    async fn get_band_musics(&self, band_id: i64) -> AppResult<Vec<Music>>;
    async fn reorder_band_musics(&self, band_id: i64, music_orders: &[(i64, i32)]) -> AppResult<()>;
    async fn get_band_member_count(&self, band_id: i64) -> AppResult<i64>;
    async fn get_band_members(&self, band_id: i64) -> AppResult<Vec<User>>;
}

#[async_trait]
pub trait ChatRepository: Send + Sync {
    // ---- Conversations ----
    async fn create_conversation(&self, conversation: &mut Conversation) -> AppResult<()>;
    async fn find_conversation(&self, id: i64) -> AppResult<Conversation>;
    async fn find_conversations_for_user(&self, user_id: i64) -> AppResult<Vec<Conversation>>;
    /// Finds an existing 1:1 private conversation between two users, if any.
    async fn find_private_between(&self, user_a: i64, user_b: i64) -> AppResult<Option<Conversation>>;
    /// Finds the conversation linked to a band, if one exists.
    async fn find_band_conversation(&self, band_id: i64) -> AppResult<Option<Conversation>>;
    async fn delete_conversation(&self, id: i64) -> AppResult<()>;

    // ---- Members ----
    async fn add_member(&self, member: &mut ConversationMember) -> AppResult<()>;
    async fn remove_member(&self, conversation_id: i64, user_id: i64) -> AppResult<()>;
    async fn find_member(
        &self,
        conversation_id: i64,
        user_id: i64,
    ) -> AppResult<Option<ConversationMember>>;
    async fn find_members(&self, conversation_id: i64) -> AppResult<Vec<ConversationMember>>;
    async fn update_member(&self, member: &ConversationMember) -> AppResult<()>;

    // ---- Messages ----
    async fn save_message(&self, message: &mut ChatMessage) -> AppResult<()>;
    async fn find_message(&self, id: i64) -> AppResult<ChatMessage>;
    async fn find_messages(
        &self,
        conversation_id: i64,
        offset: i64,
        limit: i64,
    ) -> AppResult<(Vec<ChatMessage>, i64)>;
    async fn delete_message(&self, id: i64) -> AppResult<()>;
}

#[async_trait]
pub trait ChurchRepository: Send + Sync {
    async fn create(&self, church: &mut Church) -> AppResult<()>;
    async fn find_by_id(&self, id: i64) -> AppResult<Church>;
    async fn find_by_name(&self, name: &str) -> AppResult<Church>;
    async fn find_all(&self, limit: i64, offset: i64) -> AppResult<Vec<Church>>;
    async fn find_by_denomination(&self, denomination: &str, limit: i64, offset: i64) -> AppResult<Vec<Church>>;
    async fn update(&self, church: &Church) -> AppResult<()>;
    async fn delete(&self, id: i64) -> AppResult<()>;
    async fn count(&self) -> AppResult<i64>;
    async fn count_members(&self, church_id: i64, status: &str) -> AppResult<i64>;
    async fn find_members(&self, church_id: i64, status: &str, limit: i64, offset: i64) -> AppResult<Vec<User>>;
}

#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn create(&self, notification: &mut Notification) -> AppResult<()>;
    async fn find_by_id(&self, id: i64) -> AppResult<Notification>;
    async fn find_by_user_id(&self, user_id: i64) -> AppResult<Vec<Notification>>;
    async fn find_by_user_id_paginated(
        &self,
        user_id: i64,
        offset: i64,
        limit: i64,
    ) -> AppResult<(Vec<Notification>, i64)>;
    async fn find_unread_by_user_id(&self, user_id: i64) -> AppResult<Vec<Notification>>;
    async fn get_unread_count(&self, user_id: i64) -> AppResult<i64>;
    async fn find_by_band_id(&self, band_id: i64) -> AppResult<Vec<Notification>>;
    async fn find_broadcast_notifications(&self) -> AppResult<Vec<Notification>>;
    async fn update(&self, notification: &Notification) -> AppResult<()>;
    async fn delete(&self, id: i64) -> AppResult<()>;
    async fn delete_all_by_user_id(&self, user_id: i64) -> AppResult<()>;
    async fn mark_as_read(&self, id: i64, user_id: i64) -> AppResult<()>;
    async fn mark_all_as_read(&self, user_id: i64) -> AppResult<()>;
}

#[async_trait]
pub trait DonationRepository: Send + Sync {
    async fn save(&self, donation: &mut Donation) -> AppResult<()>;
    async fn find_by_id(&self, id: i64) -> AppResult<Donation>;
    async fn find_all(&self, limit: i64, offset: i64) -> AppResult<Vec<Donation>>;
    async fn find_by_user_id(&self, user_id: i64, limit: i64, offset: i64) -> AppResult<Vec<Donation>>;
    async fn find_by_type(&self, donation_type: &str, limit: i64, offset: i64) -> AppResult<Vec<Donation>>;
    async fn find_by_donor_type(&self, donor_type: &str, limit: i64, offset: i64) -> AppResult<Vec<Donation>>;
    async fn find_by_event_id(&self, event_id: i64, limit: i64, offset: i64) -> AppResult<Vec<Donation>>;
    async fn find_by_status(&self, status: &str, limit: i64, offset: i64) -> AppResult<Vec<Donation>>;
    async fn update(&self, donation: &Donation) -> AppResult<()>;
    async fn update_status(&self, id: i64, status: &str, transaction_id: &str, payment_method: &str) -> AppResult<()>;
    async fn delete(&self, id: i64) -> AppResult<()>;
    async fn get_total_amount(&self) -> AppResult<f64>;
    async fn get_total_amount_by_type(&self, donation_type: &str) -> AppResult<f64>;
    async fn get_total_amount_by_event_id(&self, event_id: i64) -> AppResult<f64>;
    async fn count(&self) -> AppResult<i64>;
    async fn count_by_type(&self, donation_type: &str) -> AppResult<i64>;
}

#[async_trait]
pub trait SupporterRepository: Send + Sync {
    async fn create(&self, supporter: &mut Supporter) -> AppResult<()>;
    async fn find_by_id(&self, id: i64) -> AppResult<Supporter>;
    async fn find_by_email(&self, email: &str) -> AppResult<Supporter>;
    async fn find_all(&self, limit: i64, offset: i64) -> AppResult<Vec<Supporter>>;
    async fn find_by_type(&self, supporter_type: &str, limit: i64, offset: i64) -> AppResult<Vec<Supporter>>;
    async fn find_by_user(&self, user_id: i64, limit: i64, offset: i64) -> AppResult<Vec<Supporter>>;
    async fn update(&self, supporter: &Supporter) -> AppResult<()>;
    async fn delete(&self, id: i64) -> AppResult<()>;
    async fn count(&self) -> AppResult<i64>;
    async fn count_by_type(&self, supporter_type: &str) -> AppResult<i64>;
}

#[async_trait]
pub trait SettingRepository: Send + Sync {
    async fn save(&self, setting: &mut Setting) -> AppResult<()>;
    async fn find_by_user_id(&self, user_id: i64) -> AppResult<Setting>;
    async fn update(&self, setting: &mut Setting) -> AppResult<()>;
    async fn delete(&self, user_id: i64) -> AppResult<()>;
}

#[async_trait]
pub trait OtpRepository: Send + Sync {
    async fn save(&self, otp: &mut Otp) -> AppResult<()>;
    async fn find_by_email_and_purpose(&self, email: &str, purpose: &str) -> AppResult<Otp>;
    async fn find_by_email_code_and_purpose(&self, email: &str, code: &str, purpose: &str) -> AppResult<Otp>;
    async fn update(&self, otp: &Otp) -> AppResult<()>;
    async fn delete_by_email(&self, email: &str) -> AppResult<()>;
    async fn delete_by_email_and_purpose(&self, email: &str, purpose: &str) -> AppResult<()>;
    async fn delete_expired(&self) -> AppResult<()>;
}

#[async_trait]
pub trait DeviceTokenRepository: Send + Sync {
    async fn save(&self, token: &mut DeviceToken) -> AppResult<()>;
    async fn get_active_tokens_by_user_id(&self, user_id: i64) -> AppResult<Vec<String>>;
    async fn get_tokens_by_band_id(&self, band_id: i64) -> AppResult<Vec<String>>;
    async fn get_all_active_tokens(&self) -> AppResult<Vec<String>>;
    async fn get_all_active_tokens_except(&self, exclude_user_id: i64) -> AppResult<Vec<String>>;
    async fn delete_token(&self, token: &str) -> AppResult<()>;
    async fn delete_tokens(&self, tokens: &[String]) -> AppResult<()>;
    async fn deactivate_token(&self, token: &str) -> AppResult<()>;
    async fn delete_user_tokens(&self, user_id: i64) -> AppResult<()>;
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn save(&self, session: &mut Session) -> AppResult<()>;
    async fn verify(&self, user_id: i64, token: &str) -> AppResult<bool>;
    async fn find_by_user_id(&self, user_id: i64) -> AppResult<Vec<Session>>;
    async fn delete_by_id(&self, id: i64) -> AppResult<()>;
    async fn delete_by_token(&self, token: &str) -> AppResult<()>;
    async fn delete_by_user_id(&self, user_id: i64) -> AppResult<()>;
    async fn delete_by_user_id_and_device(&self, user_id: i64, device_id: &str) -> AppResult<()>;
}
