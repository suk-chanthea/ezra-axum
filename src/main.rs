//! Ezra backend — Rust/Axum port of the Go/Gin service (clean architecture).
//!
//! Composition root: loads config, connects to PostgreSQL, runs migrations, wires the
//! dependency graph (repositories → external services → use cases → handlers), and serves HTTP.

mod config;
mod domain;
mod error;
mod infrastructure;
mod interface;
mod state;
mod usecase;

use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::signal;

use crate::config::Config;
use crate::infrastructure::db;
use crate::infrastructure::email::{DummyEmailService, EmailService, SmtpEmailService};
use crate::infrastructure::fcm::{DummyFcmService, FcmService};
use crate::infrastructure::payway::{DummyPaywayService, HttpPaywayService, PaywayService};
use crate::infrastructure::persistence::band_repository::PgBandRepository;
use crate::infrastructure::persistence::booking_repository::PgBookingRepository;
use crate::infrastructure::persistence::chat_repository::PgChatRepository;
use crate::infrastructure::persistence::church_repository::PgChurchRepository;
use crate::infrastructure::persistence::device_token_repository::PgDeviceTokenRepository;
use crate::infrastructure::persistence::donation_repository::PgDonationRepository;
use crate::infrastructure::persistence::event_repository::PgEventRepository;
use crate::infrastructure::persistence::favorite_repository::PgFavoriteRepository;
use crate::infrastructure::persistence::music_repository::PgMusicRepository;
use crate::infrastructure::persistence::notification_repository::PgNotificationRepository;
use crate::infrastructure::persistence::otp_repository::PgOtpRepository;
use crate::infrastructure::persistence::setting_repository::PgSettingRepository;
use crate::infrastructure::persistence::supporter_repository::PgSupporterRepository;
use crate::infrastructure::persistence::user_repository::PgUserRepository;
use crate::infrastructure::security::jwt::JwtManager;
use crate::interface::http::router;
use crate::state::AppState;
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn".into()),
        )
        .init();

    tracing::info!("Loading configuration...");
    let cfg = Config::load()?;

    tracing::info!("Connecting to database...");
    let pool = db::new_pool(&cfg.database).await?;

    tracing::info!("Running database migrations...");
    db::run_migrations(&pool).await?;

    tracing::info!("Seeding default user (if enabled)...");
    crate::infrastructure::seed::seed_default_user(&pool).await?;

    // Repositories
    tracing::info!("Initializing repositories...");
    let user_repo = Arc::new(PgUserRepository::new(pool.clone()));
    let music_repo = Arc::new(PgMusicRepository::new(pool.clone()));
    let event_repo = Arc::new(PgEventRepository::new(pool.clone()));
    let booking_repo = Arc::new(PgBookingRepository::new(pool.clone()));
    let favorite_repo = Arc::new(PgFavoriteRepository::new(pool.clone()));
    let band_repo = Arc::new(PgBandRepository::new(pool.clone()));
    let chat_repo = Arc::new(PgChatRepository::new(pool.clone()));
    let setting_repo = Arc::new(PgSettingRepository::new(pool.clone()));
    let notification_repo = Arc::new(PgNotificationRepository::new(pool.clone()));
    let device_token_repo = Arc::new(PgDeviceTokenRepository::new(pool.clone()));
    let otp_repo = Arc::new(PgOtpRepository::new(pool.clone()));
    let donation_repo = Arc::new(PgDonationRepository::new(pool.clone()));
    let supporter_repo = Arc::new(PgSupporterRepository::new(pool.clone()));
    let church_repo = Arc::new(PgChurchRepository::new(pool.clone()));

    // External services
    let email_service: Arc<dyn EmailService> = if cfg.email.enabled {
        tracing::info!("Initializing SMTP email service...");
        Arc::new(SmtpEmailService::new(
            cfg.email.host.clone(),
            cfg.email.port.clone(),
            cfg.email.username.clone(),
            cfg.email.password.clone(),
            cfg.email.from.clone(),
            cfg.email.secure.clone(),
        ))
    } else {
        tracing::info!("Email service disabled; using dummy implementation.");
        Arc::new(DummyEmailService)
    };

    let fcm_service: Arc<dyn FcmService> = {
        if cfg.firebase.enabled && !cfg.firebase.credentials_path.is_empty() {
            tracing::warn!(
                "Firebase enabled but native FCM delivery is not implemented; using dummy service."
            );
        } else {
            tracing::info!("Firebase disabled; using dummy implementation.");
        }
        Arc::new(DummyFcmService)
    };

    let payway_service: Arc<dyn PaywayService> = if cfg.payway.enabled {
        tracing::info!("Initializing PayWay service...");
        Arc::new(HttpPaywayService::new(cfg.payway.clone()))
    } else {
        tracing::info!("PayWay disabled; using dummy implementation.");
        Arc::new(DummyPaywayService)
    };

    // Use cases
    tracing::info!("Initializing use cases...");
    let auth_uc = Arc::new(AuthUseCase::new(
        user_repo.clone(),
        otp_repo.clone(),
        JwtManager::new(&cfg.jwt.secret),
        cfg.oauth.google_client_id.clone(),
    ));
    let otp_uc = Arc::new(OtpUseCase::new(
        otp_repo.clone(),
        user_repo.clone(),
        email_service.clone(),
        cfg.jwt.token_expiry,
    ));
    let music_uc = Arc::new(MusicUseCase::new(music_repo.clone()));
    let event_uc = Arc::new(EventUseCase::new(
        event_repo.clone(),
        music_repo.clone(),
        notification_repo.clone(),
    ));
    let booking_uc = Arc::new(BookingUseCase::new(booking_repo.clone(), event_repo.clone()));
    let favorite_uc = Arc::new(FavoriteUseCase::new(favorite_repo.clone(), music_repo.clone()));
    let band_uc = Arc::new(BandUseCase::new(band_repo.clone(), music_repo.clone()));
    let chat_uc = Arc::new(ChatUseCase::new(
        chat_repo.clone(),
        band_repo.clone(),
        music_repo.clone(),
        user_repo.clone(),
        fcm_service.clone(),
    ));
    let setting_uc = Arc::new(SettingUseCase::new(setting_repo.clone()));
    let notification_uc = Arc::new(NotificationUseCase::new(
        notification_repo.clone(),
        fcm_service.clone(),
    ));
    let donation_uc = Arc::new(DonationUseCase::new(
        donation_repo.clone(),
        user_repo.clone(),
        event_repo.clone(),
        payway_service.clone(),
    ));
    let supporter_uc = Arc::new(SupporterUseCase::new(supporter_repo.clone(), donation_repo.clone()));
    let church_uc = Arc::new(ChurchUseCase::new(church_repo.clone(), user_repo.clone()));

    let state = AppState {
        auth: auth_uc,
        otp: otp_uc,
        music: music_uc,
        event: event_uc,
        booking: booking_uc,
        favorite: favorite_uc,
        band: band_uc,
        chat: chat_uc,
        setting: setting_uc,
        notification: notification_uc,
        donation: donation_uc,
        supporter: supporter_uc,
        church: church_uc,
        device_token_repo,
    };

    let app = router::build(state);

    let addr = format!("0.0.0.0:{}", cfg.app.port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("Server starting on {addr} ({})", cfg.app.environment);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Shutdown complete.");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutting down server...");
}
