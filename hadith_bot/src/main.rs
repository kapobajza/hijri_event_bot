use crate::{bot::TelegramBot, error::AppErrorKind, scheduler::Scheduler};

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

mod bot;
mod command;
mod db;
mod error;
mod scheduler;

#[tokio::main]
async fn main() -> Result<(), AppErrorKind> {
    pretty_env_logger::init();
    use bot_core::db::migrator::MigrationProject;
    use bot_core::db::migrator::Migrator;

    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        std::env::var("DB_USER").unwrap_or("postgres".to_string()),
        std::env::var("DB_PASSWORD").unwrap_or("postgres".to_string()),
        std::env::var("DB_HOST").unwrap_or("localhost".to_string()),
        std::env::var("DB_PORT").unwrap_or("5432".to_string()),
        std::env::var("DB_NAME").unwrap_or("hadith_db".to_string())
    );

    Migrator::run(&database_url, MigrationProject::HadithBot)
        .await
        .map_err(|err| {
            log::error!("Failed to run migrations: {}", err);
            AppErrorKind::MigrationError
        })?;

    let pool = sqlx::Pool::<sqlx::Postgres>::connect(&database_url)
        .await
        .map_err(|err| {
            log::error!("Failed to connect to the database: {}", err);
            AppErrorKind::DatabaseConnectionError
        })?;
    let scheduler = Scheduler::new(pool.clone()).await?;
    let bot = TelegramBot::new(pool, scheduler);

    bot.run().await;

    Ok(())
}
