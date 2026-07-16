//! Development default user seed (idempotent).

use sqlx::PgPool;

use crate::domain::entity::{Setting, User};
use crate::domain::repository::{SettingRepository, UserRepository};
use crate::infrastructure::persistence::setting_repository::PgSettingRepository;
use crate::infrastructure::persistence::user_repository::PgUserRepository;
use crate::infrastructure::security::password::hash_password;

const DEFAULT_EMAIL: &str = "ezra@gmail.com";
const DEFAULT_PASSWORD: &str = "qwer1234";
const DEFAULT_NAME: &str = "Ezra";

fn seed_enabled() -> bool {
    match std::env::var("SEED_DEFAULT_USER") {
        Ok(value) => matches!(value.to_lowercase().as_str(), "true" | "1" | "t" | "yes" | "y"),
        Err(_) => std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()) == "development",
    }
}

fn default_email() -> String {
    std::env::var("DEFAULT_USER_EMAIL").unwrap_or_else(|_| DEFAULT_EMAIL.to_string())
}

fn default_password() -> String {
    std::env::var("DEFAULT_USER_PASSWORD").unwrap_or_else(|_| DEFAULT_PASSWORD.to_string())
}

/// Creates the default local user when seeding is enabled and the account does not exist.
pub async fn seed_default_user(pool: &PgPool) -> anyhow::Result<()> {
    if !seed_enabled() {
        return Ok(());
    }

    let email = default_email();
    let user_repo = PgUserRepository::new(pool.clone());

    if user_repo.find_by_email(&email).await.is_ok() {
        tracing::info!("Default user already exists ({email}), skipping seed.");
        return Ok(());
    }

    let password = default_password();
    let hash = hash_password(&password)?;
    let mut user = User::new_local(email.clone(), DEFAULT_NAME.to_string(), email.clone(), hash);
    user.email_verified = true;
    user.role = "admin".to_string();

    user_repo.save(&mut user).await?;

    let setting_repo = PgSettingRepository::new(pool.clone());
    let mut setting = Setting::new(user.id);
    setting_repo.save(&mut setting).await?;

    tracing::info!(
        "Default user seeded: {email} (login with this email and password from DEFAULT_USER_PASSWORD)"
    );
    Ok(())
}
