use serde::{Deserialize, Serialize};

#[derive(sqlx::Type, Serialize, Deserialize, Debug)]
pub enum JobExtensionType {
    DailyHadithMessage = 1,
}

impl From<i32> for JobExtensionType {
    fn from(value: i32) -> Self {
        match value {
            1 => JobExtensionType::DailyHadithMessage,
            _ => panic!("Unknown JobExtensionType value: {}", value),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JobExtraData {
    pub extension_type: JobExtensionType,
}
