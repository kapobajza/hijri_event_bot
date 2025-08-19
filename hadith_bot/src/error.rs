use std::fmt::Display;

#[derive(Debug)]
pub enum AppErrorKind {
    SchedulerInitialization,
    ScheduleDailyHadithJob,
    GetRandomHadithFromDb,
    MigrationError,
    DatabaseConnectionError,
}

impl Display for AppErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppErrorKind::SchedulerInitialization => {
                write!(f, "Failed to initialize the scheduler")
            }
            AppErrorKind::ScheduleDailyHadithJob => {
                write!(f, "Failed to schedule daily hadith job")
            }
            AppErrorKind::GetRandomHadithFromDb => {
                write!(f, "Failed to get random hadith from database")
            }
            AppErrorKind::MigrationError => write!(f, "Database migration error"),
            AppErrorKind::DatabaseConnectionError => {
                write!(f, "Failed to connect to the database")
            }
        }
    }
}
