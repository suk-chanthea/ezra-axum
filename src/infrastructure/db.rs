//! PostgreSQL connection pool setup and migration runner (sqlx equivalent of the GORM bootstrap).

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::config::DatabaseConfig;

pub async fn new_pool(cfg: &DatabaseConfig) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&cfg.url())
        .await?;
    Ok(pool)
}

/// Runs migrations from the `migrations/` directory (mirrors GORM AutoMigrate).
pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}
