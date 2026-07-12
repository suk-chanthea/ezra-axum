//! Application configuration loaded from the environment, mirroring the Go `config` package.

use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub app: AppConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub oauth: OAuthConfig,
    pub email: EmailConfig,
    pub firebase: FirebaseConfig,
    pub payway: PaywayConfig,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub environment: String,
    pub version: String,
    pub port: String,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: String,
    pub user: String,
    pub password: String,
    pub name: String,
    pub sslmode: String,
    pub timezone: String,
}

impl DatabaseConfig {
    /// Builds a libpq/sqlx-compatible PostgreSQL connection URL.
    pub fn url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}?sslmode={}",
            self.user, self.password, self.host, self.port, self.name, self.sslmode
        )
    }
}

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub secret: String,
    /// OTP token expiry in minutes.
    pub token_expiry: i64,
}

#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub google_client_id: String,
}

#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub enabled: bool,
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub from: String,
    /// starttls (default), ssl, plain
    pub secure: String,
}

#[derive(Debug, Clone)]
pub struct FirebaseConfig {
    pub enabled: bool,
    pub credentials_path: String,
}

#[derive(Debug, Clone)]
pub struct PaywayConfig {
    pub enabled: bool,
    pub merchant_id: String,
    pub api_key: String,
    pub api_username: String,
    pub base_url: String,
    pub return_url: String,
    pub continue_url: String,
    pub callback_url: String,
}

impl Config {
    /// Reads configuration from the environment with sensible defaults.
    pub fn load() -> anyhow::Result<Self> {
        let cfg = Config {
            app: AppConfig {
                environment: get_env("APP_ENV", "development"),
                version: get_env("APP_VERSION", "1.0.0"),
                port: get_env("APP_PORT", "8080"),
            },
            database: DatabaseConfig {
                host: get_env("DB_HOST", "localhost"),
                port: get_env("DB_PORT", "5432"),
                user: get_env("DB_USER", "postgres"),
                password: get_env("DB_PASSWORD", "postgres"),
                name: get_env("DB_NAME", "ezradb"),
                sslmode: get_env("DB_SSLMODE", "disable"),
                timezone: get_env("DB_TIMEZONE", "UTC"),
            },
            jwt: JwtConfig {
                secret: get_env("JWT_SECRET", "super-secret-key"),
                token_expiry: get_env_as_int("JWT_TOKEN_EXPIRY", 10),
            },
            oauth: OAuthConfig {
                google_client_id: get_env("GOOGLE_CLIENT_ID", ""),
            },
            email: EmailConfig {
                enabled: get_env_as_bool("EMAIL_ENABLED", false),
                host: get_env("SMTP_HOST", ""),
                port: get_env("SMTP_PORT", "587"),
                username: get_env("SMTP_USERNAME", ""),
                password: get_env("SMTP_PASSWORD", ""),
                from: get_env("SMTP_FROM", ""),
                secure: get_env("SMTP_SECURE", "starttls"),
            },
            firebase: FirebaseConfig {
                enabled: get_env_as_bool("FIREBASE_ENABLED", false),
                credentials_path: get_env("FIREBASE_CREDENTIALS_PATH", ""),
            },
            payway: PaywayConfig {
                enabled: get_env_as_bool("PAYWAY_ENABLED", false),
                merchant_id: get_env("PAYWAY_MERCHANT_ID", ""),
                api_key: get_env("PAYWAY_API_KEY", ""),
                api_username: get_env("PAYWAY_API_USERNAME", ""),
                base_url: get_env("PAYWAY_BASE_URL", ""),
                return_url: get_env("PAYWAY_RETURN_URL", ""),
                continue_url: get_env("PAYWAY_CONTINUE_URL", ""),
                callback_url: get_env("PAYWAY_CALLBACK_URL", ""),
            },
        };

        if cfg.jwt.secret.is_empty() {
            anyhow::bail!("JWT_SECRET is required");
        }

        Ok(cfg)
    }
}

fn get_env(key: &str, fallback: &str) -> String {
    env::var(key).unwrap_or_else(|_| fallback.to_string())
}

fn get_env_as_bool(key: &str, fallback: bool) -> bool {
    match env::var(key) {
        Ok(value) => match value.to_lowercase().as_str() {
            "true" | "1" | "t" | "yes" | "y" => true,
            "false" | "0" | "f" | "no" | "n" => false,
            _ => fallback,
        },
        Err(_) => fallback,
    }
}

fn get_env_as_int(key: &str, fallback: i64) -> i64 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(fallback)
}
