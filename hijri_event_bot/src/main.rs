use bot_core::db::migrator::MigrationProject;

use crate::{api::HijriApi, bot::TelegramBot, i18n::instance::I18n, scheduler::Scheduler};

mod api;
mod bot;
mod command;
mod error;
mod i18n;
mod scheduler;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    use bot_core::db::migrator::Migrator;
    use std::sync::Arc;

    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        std::env::var("DB_USER").unwrap_or("postgres".to_string()),
        std::env::var("DB_PASSWORD").unwrap_or("postgres".to_string()),
        std::env::var("DB_HOST").unwrap_or("localhost".to_string()),
        std::env::var("DB_PORT").unwrap_or("5433".to_string()),
        std::env::var("DB_NAME").unwrap_or("hijri_db".to_string())
    );

    Migrator::run(&database_url, MigrationProject::HijriEventBot).await?;

    let i18n = Arc::new(I18n::new().expect("Failed to initialize i18n"));
    let api = Arc::new(HijriApi::new(i18n.clone()));
    let pool = sqlx::Pool::<sqlx::Postgres>::connect(&database_url).await?;
    let scheduler = Scheduler::new(pool.clone(), api.clone(), i18n.clone()).await?;

    let bot = TelegramBot::new(api, i18n, pool, scheduler);

    bot.run().await;

    log::info!("Hijri bot has stopped.");

    Ok(())
}
