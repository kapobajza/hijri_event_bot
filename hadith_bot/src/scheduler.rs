use std::sync::Arc;

use bot_core::{
    bot_core::BotCore,
    db::{
        job::{JobExtensionType, JobExtraData},
        postgres_metadata_store::PostgresMetadataStore,
        postgres_notification_store::PostgresNotificationStore,
    },
};
use sqlx::{Pool, Postgres, types::uuid};
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
        chat_id: ChatId,
        user_id: uuid::Uuid,
    ) -> Result<(), AppErrorKind> {
        let bot = Arc::new(bot);
        let hadith_repo = Arc::clone(&self.hadith_repo);

        let mut daily_hadith_job = Job::new_async("0 0 8 * * *", move |_uuid, _l| {
            let bot = bot.clone();
            let hadith_repo = hadith_repo.clone();

            Box::pin(async move {
                match hadith_repo.get_random_hadith_text().await {
                    Ok(hadith) => {
                        BotCore::send_message(&bot, chat_id, hadith).await;
                    }
                    Err(e) => {
                        log::error!("Failed to fetch daily hadith: {}", e);
                        BotCore::send_message(
                            &bot,
                            chat_id,
                            "Dogodila se greška. Pokušajte kasnije.".to_string(),
                        )
                        .await;
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
            user_id,
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
