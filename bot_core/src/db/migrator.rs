pub struct Migrator;

pub enum MigrationProject {
    HijriEventBot,
    HadithBot,
}

impl Migrator {
    pub async fn run(
        database_url: &str,
        migration_project: MigrationProject,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use sqlx::postgres::PgPoolOptions;

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        // Use a match to support different migration directories at compile time
        match migration_project {
            MigrationProject::HijriEventBot => {
                sqlx::migrate!("../hijri_event_bot/migrations")
                    .run(&pool)
                    .await?
            }
            MigrationProject::HadithBot => {
                sqlx::migrate!("../hadith_bot/migrations")
                    .run(&pool)
                    .await?
            }
        }

        log::info!("Database migrations completed successfully");

        Ok(())
    }
}
