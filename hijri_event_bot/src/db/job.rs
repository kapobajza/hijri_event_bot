use bot_core::db::postgres_metadata_store::JobCallbacksExtension;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sqlx::{Pool, Postgres, Transaction, types::uuid};

#[derive(sqlx::Type, Serialize, Deserialize, Debug)]
pub enum JobExtensionType {
    WhiteDaysMessage = 1,
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

pub struct JobCallbacks;

impl JobCallbacksExtension for JobCallbacks {
    fn before_job_add<'a>(
        &'a self,
        job: &'a bot_core::db::tables::Job,
        pool: &'a Pool<Postgres>,
    ) -> std::pin::Pin<
        Box<dyn Future<Output = Result<bool, tokio_cron_scheduler::JobSchedulerError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let extra_job_data = job.extra.clone().ok_or_else(|| {
                log::error!("Job extra data is missing");
                tokio_cron_scheduler::JobSchedulerError::CantAdd
            })?;
            let extra_job_data: JobExtraData =
                serde_json::from_slice(&extra_job_data).map_err(|err| {
                    log::error!("Failed to deserialize job extra data: {}", err);
                    tokio_cron_scheduler::JobSchedulerError::CantAdd
                })?;

            let user_has_job = sqlx::query_scalar!(
                "
                    SELECT EXISTS(
                        SELECT 1 FROM job_extensions AS je
                        JOIN users_jobs AS uj ON uj.job_id = je.job_id
                        WHERE je.type = $1 AND user_id = $2 LIMIT 1
                    )
                ",
                JobExtensionType::WhiteDaysMessage as i32,
                extra_job_data.user_id
            )
            .fetch_one(pool)
            .await
            .map_err(|e| {
                log::error!(
                    "Failed to check if user has white days message job. user id: {}, error: {}",
                    extra_job_data.user_id,
                    e
                );
                tokio_cron_scheduler::JobSchedulerError::CantAdd
            })?;

            if user_has_job.unwrap_or(false) {
                log::info!("User already has white days message job, skipping");
                return Ok(true);
            }

            Ok(false)
        })
    }

    fn after_job_add<'a, 'tx>(
        &'a self,
        job: &'a bot_core::db::tables::Job,
        tx: &'a mut Transaction<'tx, Postgres>,
    ) -> std::pin::Pin<
        Box<dyn Future<Output = Result<(), tokio_cron_scheduler::JobSchedulerError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let extra_job_data = job.extra.clone().ok_or_else(|| {
                log::error!("Job extra data is missing");
                tokio_cron_scheduler::JobSchedulerError::CantAdd
            })?;
            let extra_job_data: JobExtraData =
                serde_json::from_slice(&extra_job_data).map_err(|err| {
                    log::error!("Failed to deserialize job extra data: {}", err);
                    tokio_cron_scheduler::JobSchedulerError::CantAdd
                })?;

            sqlx::query!(
                "INSERT INTO job_extensions (job_id, type) VALUES ($1, $2)",
                job.id,
                extra_job_data.extension_type as i32
            )
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                log::error!(
                    "Failed to insert job extension for job id: {}, error: {}",
                    job.id,
                    e
                );
                tokio_cron_scheduler::JobSchedulerError::CantAdd
            })?;

            sqlx::query!(
                "INSERT INTO users_jobs (id, job_id, user_id) VALUES ($1, $2, $3)",
                uuid::Uuid::new_v4(),
                job.id,
                extra_job_data.user_id
            )
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                log::error!(
                    "Failed to insert user job. job id: {}, user id: {}, error: {}",
                    job.id,
                    extra_job_data.user_id,
                    e
                );
                tokio_cron_scheduler::JobSchedulerError::CantAdd
            })?;

            Ok(())
        })
    }
}
