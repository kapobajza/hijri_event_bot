use std::{pin::Pin, sync::Arc};

use sqlx::{Pool, Postgres, Transaction, types::uuid};
use tokio_cron_scheduler::{
    JobSchedulerError, MetaDataStorage,
    job::job_data_prost::JobStoredData,
    store::{DataStore, InitStore},
};

use crate::db::{job::JobExtraData, tables::Job};

pub trait JobCallbacksExtension: Send + Sync + 'static {
    fn before_job_add<'a>(
        &'a self,
        job: &'a Job,
        pool: &'a Pool<Postgres>,
    ) -> Pin<Box<dyn Future<Output = Result<bool, JobSchedulerError>> + Send + 'a>>;

    fn after_job_add<'a, 'tx>(
        &'a self,
        job: &'a Job,
        tx: &'a mut Transaction<'tx, Postgres>,
    ) -> Pin<Box<dyn Future<Output = Result<(), JobSchedulerError>> + Send + 'a>>;
}

pub struct PostgresMetadataStore {
    pool: sqlx::Pool<sqlx::Postgres>,
    callbacks: Option<Arc<dyn JobCallbacksExtension>>,
}

impl PostgresMetadataStore {
    pub fn new(pool: sqlx::Pool<sqlx::Postgres>) -> Self {
        Self {
            pool,
            callbacks: None,
        }
    }

    pub fn with_callbacks(mut self, callbacks: Arc<dyn JobCallbacksExtension>) -> Self {
        self.callbacks = Some(callbacks);
        self
    }
}

impl DataStore<JobStoredData> for PostgresMetadataStore {
    fn get(
        &mut self,
        id: uuid::Uuid,
    ) -> std::pin::Pin<
        Box<
            dyn Future<
                    Output = Result<Option<JobStoredData>, tokio_cron_scheduler::JobSchedulerError>,
                > + Send,
        >,
    > {
        let pool = self.pool.clone();

        Box::pin(async move {
            let job = sqlx::query_as!(
                Job,
                "
                SELECT  
                    id,
                    last_updated,
                    next_tick,
                    last_tick,
                    job_type,
                    count,
                    ran,
                    stopped,
                    time_offset_seconds,
                    extra,
                    schedule,
                    repeated_every,
                    repeating
                FROM jobs WHERE id = $1
                LIMIT 1
                ",
                id,
            )
            .fetch_optional(&pool)
            .await
            .map_err(|e| {
                log::error!("Failed to fetch job: {}", e);
                tokio_cron_scheduler::JobSchedulerError::FetchJob
            })?;

            match job {
                Some(job) => Ok(job.into()),
                None => {
                    log::error!("Job with id {} not found", id);
                    Err(tokio_cron_scheduler::JobSchedulerError::FetchJob)
                }
            }
        })
    }

    fn add_or_update(
        &mut self,
        data: JobStoredData,
    ) -> std::pin::Pin<
        Box<dyn Future<Output = Result<(), tokio_cron_scheduler::JobSchedulerError>> + Send>,
    > {
        let pool = self.pool.clone();

        Box::pin(async move {
            let job: Job = data.into();

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
                extra_job_data.extension_type as i32,
                extra_job_data.user_id
            )
            .fetch_one(&pool)
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
                log::info!("User already has job scheduled, skipping");
                return Ok(());
            }

            let mut tx = pool.begin().await.map_err(|e| {
                log::error!("Failed to begin transaction: {}", e);
                tokio_cron_scheduler::JobSchedulerError::CantAdd
            })?;

            match sqlx::query!(
                "
                INSERT INTO jobs (id, last_updated, next_tick, last_tick, job_type, count, ran, stopped, schedule, repeating, repeated_every, time_offset_seconds, extra) \
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                ON CONFLICT (id) DO UPDATE SET \
                last_updated = EXCLUDED.last_updated,
                next_tick = EXCLUDED.next_tick,
                last_tick = EXCLUDED.last_tick,
                job_type = EXCLUDED.job_type,
                count = EXCLUDED.count,
                ran = EXCLUDED.ran,
                stopped = EXCLUDED.stopped,
                schedule = EXCLUDED.schedule,
                repeating = EXCLUDED.repeating,
                repeated_every = EXCLUDED.repeated_every,
                time_offset_seconds = EXCLUDED.time_offset_seconds,
                extra = EXCLUDED.extra
                ",
                job.id,
                job.last_updated,
                job.next_tick,
                job.last_tick,
                job.job_type,
                job.count,
                job.ran,
                job.stopped,
                job.schedule.as_deref(),
                job.repeating,
                job.repeated_every.map(|i| i as i32),
                job.time_offset_seconds,
                job.extra,
            ).execute(&mut *tx)
            .await {
                Ok(_) => {}
                Err(e) => {
                    tx.rollback().await.map_err(|e| {
                        log::error!("Failed to rollback transaction: {}", e);
                        tokio_cron_scheduler::JobSchedulerError::CantAdd
                    })?;
                    log::error!("Failed to insert or update job: {}, error: {}", job.id, e);
                    return Err(tokio_cron_scheduler::JobSchedulerError::CantAdd);
                }
            }

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
            .execute(&mut *tx)
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
            .execute(&mut *tx)
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

            tx.commit().await.map_err(|e| {
                log::error!("Failed to commit transaction: {}", e);
                tokio_cron_scheduler::JobSchedulerError::CantAdd
            })?;

            Ok(())
        })
    }

    fn delete(
        &mut self,
        guid: uuid::Uuid,
    ) -> std::pin::Pin<
        Box<dyn Future<Output = Result<(), tokio_cron_scheduler::JobSchedulerError>> + Send>,
    > {
        let pool = self.pool.clone();

        Box::pin(async move {
            sqlx::query!("DELETE FROM jobs WHERE id = $1", guid)
                .execute(&pool)
                .await
                .map_err(|e| {
                    log::error!("Failed to delete job: {}", e);
                    tokio_cron_scheduler::JobSchedulerError::CantRemove
                })?;

            Ok(())
        })
    }
}

impl InitStore for PostgresMetadataStore {
    fn init(
        &mut self,
    ) -> std::pin::Pin<
        Box<dyn Future<Output = Result<(), tokio_cron_scheduler::JobSchedulerError>> + Send>,
    > {
        Box::pin(async { Ok(()) })
    }

    fn inited(
        &mut self,
    ) -> std::pin::Pin<
        Box<dyn Future<Output = Result<bool, tokio_cron_scheduler::JobSchedulerError>> + Send>,
    > {
        Box::pin(async { Ok(true) })
    }
}

impl MetaDataStorage for PostgresMetadataStore {
    fn list_next_ticks(
        &mut self,
    ) -> std::pin::Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Vec<tokio_cron_scheduler::job::job_data_prost::JobAndNextTick>,
                        tokio_cron_scheduler::JobSchedulerError,
                    >,
                > + Send,
        >,
    > {
        let pool = self.pool.clone();

        Box::pin(async move {
            let rows = sqlx::query!(
                "SELECT id, next_tick, job_type, last_tick \
                      FROM jobs \
                      WHERE next_tick > 0 AND next_tick <= $1",
                chrono::Utc::now().timestamp()
            )
            .fetch_all(&pool)
            .await
            .map_err(|e| {
                log::error!("Failed to list next ticks: {}", e);
                tokio_cron_scheduler::JobSchedulerError::FetchJob
            })?;

            Ok(rows
                .into_iter()
                .map(
                    |row| tokio_cron_scheduler::job::job_data_prost::JobAndNextTick {
                        id: Some(row.id.into()),
                        next_tick: row.next_tick.map(|i| i as u64).unwrap_or_default(),
                        job_type: row.job_type,
                        last_tick: row.last_tick.map(|i| i as u64),
                    },
                )
                .collect())
        })
    }

    fn set_next_and_last_tick(
        &mut self,
        guid: uuid::Uuid,
        next_tick: Option<chrono::DateTime<chrono::Utc>>,
        last_tick: Option<chrono::DateTime<chrono::Utc>>,
    ) -> std::pin::Pin<
        Box<dyn Future<Output = Result<(), tokio_cron_scheduler::JobSchedulerError>> + Send>,
    > {
        let pool = self.pool.clone();

        Box::pin(async move {
            let next_tick = next_tick.map(|b| b.timestamp()).unwrap_or(0);
            let last_tick = last_tick.map(|b| b.timestamp());

            sqlx::query!(
                "UPDATE jobs SET next_tick = $1, last_tick = $2 WHERE id = $3",
                next_tick,
                last_tick,
                guid
            )
            .execute(&pool)
            .await
            .map_err(|e| {
                log::error!("Failed to set next and last tick: {}", e);
                tokio_cron_scheduler::JobSchedulerError::UpdateJobData
            })?;

            Ok(())
        })
    }

    fn time_till_next_job(
        &mut self,
    ) -> std::pin::Pin<
        Box<
            dyn Future<
                    Output = Result<
                        Option<std::time::Duration>,
                        tokio_cron_scheduler::JobSchedulerError,
                    >,
                > + Send,
        >,
    > {
        let pool = self.pool.clone();

        Box::pin(async move {
            let now = chrono::Utc::now().timestamp();
            let row = sqlx::query!(
                "SELECT next_tick \
                      FROM jobs \
                      WHERE next_tick > 0 AND next_tick > $1 \
                      ORDER BY next_tick ASC \
                      LIMIT 1",
                now
            )
            .fetch_one(&pool)
            .await
            .map_err(|e| {
                log::error!("Failed to fetch next job time: {}", e);
                tokio_cron_scheduler::JobSchedulerError::FetchJob
            })?;

            Ok(row
                .next_tick
                .map(|ts| ts - now)
                .filter(|ts| *ts > 0)
                .map(|ts| ts as u64)
                .map(std::time::Duration::from_secs))
        })
    }
}
