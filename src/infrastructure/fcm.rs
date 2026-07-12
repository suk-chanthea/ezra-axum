//! Firebase Cloud Messaging abstraction, mirroring the Go `infrastructure/firebase` package.
//!
//! Real delivery requires Firebase service-account credentials. As in the Go service, when
//! credentials are not configured a no-op (dummy) implementation is used, which is the default.

use std::collections::HashMap;

use async_trait::async_trait;

use crate::error::AppResult;

#[async_trait]
pub trait FcmService: Send + Sync {
    async fn send_to_user(&self, user_id: i64, title: &str, body: &str, data: HashMap<String, String>) -> AppResult<()>;
    async fn send_to_band(&self, band_id: i64, title: &str, body: &str, data: HashMap<String, String>) -> AppResult<()>;
    async fn send_to_all_except(&self, exclude_user_id: i64, title: &str, body: &str, data: HashMap<String, String>) -> AppResult<()>;
}

pub struct DummyFcmService;

#[async_trait]
impl FcmService for DummyFcmService {
    async fn send_to_user(&self, user_id: i64, title: &str, body: &str, _data: HashMap<String, String>) -> AppResult<()> {
        tracing::info!("[DUMMY FCM] Would send to user {user_id}: {title} - {body}");
        Ok(())
    }

    async fn send_to_band(&self, band_id: i64, title: &str, body: &str, _data: HashMap<String, String>) -> AppResult<()> {
        tracing::info!("[DUMMY FCM] Would send to band {band_id}: {title} - {body}");
        Ok(())
    }

    async fn send_to_all_except(&self, exclude_user_id: i64, title: &str, body: &str, _data: HashMap<String, String>) -> AppResult<()> {
        tracing::info!("[DUMMY FCM] Would broadcast (except user {exclude_user_id}): {title} - {body}");
        Ok(())
    }
}
