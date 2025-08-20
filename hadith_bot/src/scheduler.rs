use std::sync::Arc;

use bot_core::{
    bot_core::BotCore,
    db::{
        job::{JobExtensionType, JobExtraData},
        postgres_metadata_store::PostgresMetadataStore,
        postgres_notification_store::PostgresNotificationStore,
    },
};
use sqlx::{Pool, Postgres};
use teloxide::{Bot, types::ChatId};
use tokio_cron_scheduler::{
    Job, JobScheduler, SimpleJobCode, SimpleNotificationCode, job::job_data_prost::JobStoredData,
};

use crate::{db::HadithRepository, error::AppErrorKind};

pub struct Scheduler {
    sched: JobScheduler,
    hadith_repo: Arc<HadithRepository>,
}

impl Scheduler {
    pub async fn new(pool: Pool<Postgres>) -> Result<Self, AppErrorKind> {
        let mut sched = JobScheduler::new_with_storage_and_code(
            Box::new(PostgresMetadataStore::new(pool.clone())),
            Box::new(PostgresNotificationStore::new(pool.clone())),
            Box::new(SimpleJobCode::default()),
            Box::new(SimpleNotificationCode::default()),
            200,
        )
        .await
        .map_err(|err| {
            log::error!("Failed to create JobScheduler: {}", err);
            AppErrorKind::SchedulerInitialization
        })?;

        sched.shutdown_on_ctrl_c();

        sched.set_shutdown_handler(Box::new(|| {
            Box::pin(async move {
                info!("Shut down done");
            })
        }));

        sched.start().await.map_err(|err| {
            log::error!("Failed to start JobScheduler: {}", err);
            AppErrorKind::SchedulerInitialization
        })?;

        Ok(Self {
            sched,
            hadith_repo: Arc::new(HadithRepository::new(pool)),
        })
    }

    pub async fn schedule_daily_hadith_job(
        &self,
        bot: Bot,
        pool: Arc<Pool<Postgres>>,
    ) -> Result<(), AppErrorKind> {
        let bot = Arc::new(bot);
        let hadith_repo = Arc::clone(&self.hadith_repo);

        let job_with_type_exists = sqlx::query_scalar!(
            "
                SELECT EXISTS (
                    SELECT 1 
                    FROM job_extensions 
                    WHERE type = $1
                )
            ",
            JobExtensionType::DailyHadithMessage as i32,
        )
        .fetch_one(&*pool)
        .await
        .map_err(|e| {
            log::error!("Failed to check for existing job: {}", e);
            AppErrorKind::ScheduleDailyHadithJob
        })?;

        if job_with_type_exists.unwrap_or(false) {
            log::info!(
                "Job with type {:?} already exists, skipping creation.",
                JobExtensionType::DailyHadithMessage
            );
            return Ok(());
        }

        let mut daily_hadith_job = Job::new_async("0 58 19 * * *", move |_uuid, _l| {
            let bot = bot.clone();
            let hadith_repo = hadith_repo.clone();
            let pool = pool.clone();

            Box::pin(async move {
                let hadith_repo = hadith_repo.clone();
                let bot = bot.clone();

                let chat_handles_res = sqlx::query!("SELECT chat_id FROM users")
                    .fetch_all(&*pool)
                    .await
                    .map_err(|e| {
                        log::error!("Failed to fetch users: {}", e);
                        AppErrorKind::SendDailyHadithMessage
                    })
                    .map(|rows| {
                        let rows: Vec<i64> = rows.into_iter().map(|row| row.chat_id).collect();

                        rows.into_iter().map(|chat_id| {
                            let hadith_repo = hadith_repo.clone();
                            let bot = bot.clone();

                            tokio::spawn(async move {
                                let chat_id = ChatId(chat_id);
                                match hadith_repo.get_random_hadith_text().await {
                                    Ok(hadith) => {
                                        BotCore::send_message(&bot, chat_id, hadith).await;
                                    }
                                    Err(e) => {
                                        log::error!("Failed to fetch daily hadith: {}", e);
                                        BotCore::send_message(
                                            &bot,
                                            chat_id,
                                            "Dogodila se greška. Pokušajte ponovo kasnije."
                                                .to_string(),
                                        )
                                        .await;
                                    }
                                }
                            })
                        })
                    });

                match chat_handles_res {
                    Ok(handles) => {
                        let len = handles.len();

                        for handle in handles {
                            if let Err(e) = handle.await {
                                log::error!("Error in daily hadith job: {}", e);
                            }
                        }

                        log::info!("Successfully sent {} daily hadith messages", len);
                    }
                    Err(e) => {
                        log::error!("Error fetching chat handles: {}", e);
                    }
                }
            })
        })
        .map_err(|err| {
            log::error!("Failed to create daily hadith job: {}", err);
            AppErrorKind::ScheduleDailyHadithJob
        })?;

        let job_data = daily_hadith_job.job_data().map_err(|err| {
            log::error!("Failed to get job data: {}", err);
            AppErrorKind::ScheduleDailyHadithJob
        })?;

        let extra_data = serde_json::to_vec(&JobExtraData {
            extension_type: JobExtensionType::DailyHadithMessage,
        })
        .map_err(|err| {
            log::error!("Failed to serialize job extra data: {}", err);
            AppErrorKind::ScheduleDailyHadithJob
        })?;

        daily_hadith_job
            .set_job_data(JobStoredData {
                extra: extra_data,
                ..job_data
            })
            .map_err(|err| {
                log::error!("Failed to set job data: {}", err);
                AppErrorKind::ScheduleDailyHadithJob
            })?;

        self.sched.add(daily_hadith_job).await.map_err(|err| {
            log::error!("Failed to schedule daily hadith job: {}", err);
            AppErrorKind::ScheduleDailyHadithJob
        })?;

        Ok(())
    }
}
