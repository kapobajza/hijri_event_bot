use std::backtrace::Backtrace;

pub struct AppError {
    pub actual_error: String,
    pub message_translation_key: String,
    backtrace: Option<Backtrace>,
}

impl AppError {
    pub fn new(actual_error: String, message_translation_key: &str) -> Self {
        let backtrace = if cfg!(debug_assertions) || std::env::var("RUST_BACKTRACE").is_ok() {
            Some(Backtrace::capture())
        } else {
            None
        };

        error!("AppError: {}", actual_error,);

        AppError {
            actual_error,
            message_translation_key: message_translation_key.to_string(),
            backtrace,
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AppError: {}, Translation Key: {}",
            self.actual_error, self.message_translation_key
        )?;

        if let Some(backtrace) = &self.backtrace {
            write!(f, "\nBacktrace: {}", backtrace)?;
        }

        Ok(())
    }
}
