use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sqlx::types::uuid;

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

fn serialize_uuid<S>(uuid: &uuid::Uuid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&uuid.to_string())
}

fn deserialize_uuid<'de, D>(deserializer: D) -> Result<uuid::Uuid, D::Error>
where
    D: Deserializer<'de>,
{
    let uuid_str = String::deserialize(deserializer)?;
    uuid::Uuid::parse_str(&uuid_str).map_err(serde::de::Error::custom)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JobExtraData {
    #[serde(
        serialize_with = "serialize_uuid",
        deserialize_with = "deserialize_uuid"
    )]
    pub user_id: uuid::Uuid,
    pub extension_type: JobExtensionType,
}
