//! Shared application state (dependency-injection container), mirroring the Go `cmd/main` wiring.

use std::sync::Arc;

use crate::domain::repository::DeviceTokenRepository;
use crate::usecase::auth::AuthUseCase;
use crate::usecase::band::BandUseCase;
use crate::usecase::booking::BookingUseCase;
use crate::usecase::chat::ChatUseCase;
use crate::usecase::church::ChurchUseCase;
use crate::usecase::donation::DonationUseCase;
use crate::usecase::event::EventUseCase;
use crate::usecase::favorite::FavoriteUseCase;
use crate::usecase::music::MusicUseCase;
use crate::usecase::notification::NotificationUseCase;
use crate::usecase::otp::OtpUseCase;
use crate::usecase::setting::SettingUseCase;
use crate::usecase::supporter::SupporterUseCase;

#[derive(Clone)]
pub struct AppState {
    pub auth: Arc<AuthUseCase>,
    pub otp: Arc<OtpUseCase>,
    pub music: Arc<MusicUseCase>,
    pub event: Arc<EventUseCase>,
    pub booking: Arc<BookingUseCase>,
    pub favorite: Arc<FavoriteUseCase>,
    pub band: Arc<BandUseCase>,
    pub chat: Arc<ChatUseCase>,
    pub setting: Arc<SettingUseCase>,
    pub notification: Arc<NotificationUseCase>,
    pub donation: Arc<DonationUseCase>,
    pub supporter: Arc<SupporterUseCase>,
    pub church: Arc<ChurchUseCase>,
    pub device_token_repo: Arc<dyn DeviceTokenRepository>,
}
