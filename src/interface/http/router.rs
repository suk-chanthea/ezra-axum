//! HTTP router wiring (mirrors the Go `interface/http/router` package).
//!
//! Public routes are mounted at the root; authenticated routes live under `/api` and use the
//! `AuthUser` extractor (the equivalent of the Gin `JWTMiddleware`).

use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use serde_json::json;

use super::handler::{
    auth, band, booking, chat, church, device_token, donation, event, favorite, music,
    notification, otp, setting, supporter,
};
use crate::state::AppState;

use tower_http::cors::{CorsLayer, Any};

pub fn build(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .merge(public_routes())
        .nest("/api", protected_routes())
        .layer(cors)
        .with_state(state)
}

fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/ping", get(|| async { Json(json!({ "message": "api work..." })) }))
        // Auth & OTP
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .route("/auth/google", post(auth::google_login))
        .route("/auth/reset-password", post(auth::reset_password))
        .route("/otp/send", post(otp::send_otp))
        .route("/otp/verify", post(otp::verify_otp))
        // Public donations
        .route("/donations", get(donation::get_all))
        .route("/donations/stats", get(donation::get_stats))
        .route("/donations/:id", get(donation::get_by_id))
        .route("/donations/:id/pay", post(donation::initiate_payment))
        .route("/donations/:id/regenerate-qr", post(donation::regenerate_qr))
        .route("/donations/:id/status", get(donation::check_qr_status))
        .route("/donations/type/:type", get(donation::get_by_type))
        .route("/donations/event/:event_id", get(donation::get_by_event))
        .route("/donations/event/:event_id/stats", get(donation::get_stats_by_event))
        .route("/webhooks/payway", post(donation::handle_payway_webhook))
        // Public supporters
        .route("/supporters", get(supporter::get_all))
        .route("/supporters/search", get(supporter::get_by_email))
        .route("/supporters/type/:type", get(supporter::get_by_type))
        .route("/supporters/:id", get(supporter::get_by_id))
        // Public churches
        .route("/churches", get(church::get_all))
        .route("/churches/denomination/:denomination", get(church::get_by_denomination))
        .route("/churches/:id", get(church::get_by_id))
        .route("/churches/:id/members", get(church::get_members))
}

fn protected_routes() -> Router<AppState> {
    Router::new()
        // User / auth
        .route("/me", get(auth::get_me).put(auth::update_me))
        .route("/logout", post(auth::logout))
        .route("/logout/all", post(auth::logout_all))
        .route("/user", delete(auth::delete_user))
        .route("/sessions", get(auth::get_sessions))
        .route("/sessions/:id", delete(auth::revoke_session))
        // Music
        .route("/musics", get(music::get_all).post(music::create))
        .route("/musics/user", get(music::get_by_user))
        .route("/musics/:id", get(music::get_by_id).put(music::update).delete(music::delete))
        // Event
        .route("/events", get(event::get_all).post(event::create))
        .route("/events/user", get(event::get_by_user))
        .route("/events/:id", get(event::get_by_id).put(event::update).delete(event::delete))
        // Booking
        .route("/bookings", get(booking::get_all).post(booking::create))
        .route("/bookings/user", get(booking::get_by_user))
        .route("/bookings/event/:event_id", get(booking::get_by_event))
        .route("/bookings/:id", get(booking::get_by_id).put(booking::update).delete(booking::delete))
        // Favorite
        .route("/favorites", get(favorite::get_user_favorites))
        .route("/favorites/music/:id", post(favorite::add_favorite).delete(favorite::remove_favorite))
        .route("/favorites/music/:id/check", get(favorite::is_favorite))
        .route("/favorites/music/:id/count", get(favorite::get_favorite_count))
        // Band
        .route("/bands", get(band::get_all).post(band::create))
        .route("/bands/user", get(band::get_by_user))
        .route("/bands/public", get(band::get_public))
        .route("/bands/:id", get(band::get_by_id).put(band::update).delete(band::delete))
        .route("/bands/:id/musics", get(band::get_musics).post(band::add_musics))
        .route("/bands/:id/musics/reorder", put(band::reorder_musics))
        .route("/bands/:id/musics/:music_id", delete(band::remove_music))
        .route("/bands/:id/members", get(band::get_members))
        // Chat — conversations (private / group / band)
        .route("/chat/conversations", get(chat::get_conversations).post(chat::create_conversation))
        .route("/chat/conversations/:id", get(chat::get_conversation).delete(chat::delete_conversation))
        // Chat — members & permissions
        .route("/chat/conversations/:id/members", get(chat::get_members).post(chat::add_member))
        .route(
            "/chat/conversations/:id/members/:member_id",
            put(chat::update_member).delete(chat::remove_member),
        )
        // Chat — messages (text / voice / image / shared songs)
        .route("/chat/conversations/:id/messages", get(chat::get_messages).post(chat::send_message))
        .route(
            "/chat/conversations/:id/messages/:message_id",
            get(chat::get_message).delete(chat::delete_message),
        )
        // Setting
        .route("/settings", get(setting::get_settings).put(setting::update_settings))
        .route("/settings/reset", post(setting::reset_settings))
        // Notification
        .route("/notifications", get(notification::get_all).post(notification::create).delete(notification::delete_all))
        .route("/notifications/band/:band_id", post(notification::create_band_notification))
        .route("/notifications/broadcast", post(notification::create_broadcast))
        .route("/notifications/unread", get(notification::get_unread))
        .route("/notifications/unread/count", get(notification::get_unread_count))
        .route("/notifications/read-all", put(notification::mark_all_as_read))
        .route("/notifications/:id", get(notification::get_by_id).delete(notification::delete))
        .route("/notifications/:id/read", put(notification::mark_as_read))
        // Device tokens
        .route("/device-tokens/register", post(device_token::register_token))
        .route("/device-tokens/unregister", post(device_token::unregister_token))
        .route("/device-tokens/clear", delete(device_token::delete_all_tokens))
        // Protected donations
        .route("/donations", post(donation::create))
        .route("/donations/user", get(donation::get_by_user))
        .route("/donations/:id/status", put(donation::update_status))
        .route("/donations/:id", delete(donation::delete))
        // Protected supporters
        .route("/supporters", post(supporter::create))
        .route("/supporters/user", get(supporter::get_by_user))
        .route("/supporters/:id/stats", get(supporter::get_stats))
        .route("/supporters/:id", put(supporter::update).delete(supporter::delete))
        // Protected churches
        .route("/churches", post(church::create))
        .route("/churches/join", post(church::join_church))
        .route("/churches/leave", post(church::leave_church))
        .route("/churches/:id", put(church::update).delete(church::delete))
        .route("/churches/:id/pending", get(church::get_pending_members))
        .route("/churches/:id/approve", post(church::approve_member))
}
