pub struct Migrator;

impl Migrator {
    pub async fn run(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
        use sqlx::postgres::PgPoolOptions;

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        log::info!("Database migrations completed successfully");

        Ok(())
    }
}
