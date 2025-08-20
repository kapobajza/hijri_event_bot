use serde::{Deserialize, Serialize};

#[derive(sqlx::Type, Serialize, Deserialize, Debug)]
pub enum JobExtensionType {
    WhiteDaysMessage = 1,
    DailyHadithMessage = 2,
}

impl From<i32> for JobExtensionType {
    fn from(value: i32) -> Self {
        match value {
            1 => JobExtensionType::WhiteDaysMessage,
            _ => panic!("Unknown JobExtensionType value: {}", value),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JobExtraData {
    pub extension_type: JobExtensionType,
}
