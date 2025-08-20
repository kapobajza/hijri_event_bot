use std::{collections::HashMap, sync::Arc};

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

use crate::{
    api::HijriApi,
    error::AppErrorKind,
    i18n::{instance::I18n, translation_key::TranslationKey},
};

pub struct Scheduler {
    api: Arc<HijriApi>,
    sched: JobScheduler,
    i18n: Arc<I18n>,
}

const DAY_BEFORE_FIRST_WHITE_DAY: u8 = 12;

impl Scheduler {
    pub async fn new(
        pool: Pool<Postgres>,
        api: Arc<HijriApi>,
        i18n: Arc<I18n>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let postgres_metadata_store = PostgresMetadataStore::new(pool.clone());

        let mut sched = JobScheduler::new_with_storage_and_code(
            Box::new(postgres_metadata_store),
            Box::new(PostgresNotificationStore::new(pool)),
            Box::new(SimpleJobCode::default()),
            Box::new(SimpleNotificationCode::default()),
            200,
        )
        .await?;

        sched.shutdown_on_ctrl_c();

        sched.set_shutdown_handler(Box::new(|| {
            Box::pin(async move {
                info!("Shut down done");
            })
        }));

        sched.start().await?;

        Ok(Self { sched, api, i18n })
    }

    pub async fn schedule_white_days_message(
        &self,
        bot: Bot,
        chat_id: i64,
    ) -> Result<(), AppErrorKind> {
        let api = Arc::clone(&self.api);
        let i18n = Arc::clone(&self.i18n);
        let bot = Arc::new(bot);

        // This job will run at 18:00 every day to check if it's the 12th of the month
        // and send a notification if it is
        let mut white_days_message_job = Job::new_async("0 0 18 * * *", move |_uuid, _l| {
            let api = Arc::clone(&api);
            let i18n = Arc::clone(&i18n);
            let bot = Arc::clone(&bot);

            Box::pin(async move {
                let current_date = api
                    .get_current_hijri_date()
                    .await
                    .map_err(|_err| AppErrorKind::WhiteDaysMessage);

                match current_date {
                    Ok(date_response) => {
                        if date_response.day_number == DAY_BEFORE_FIRST_WHITE_DAY
                            // We will skip the notification for the month of Ramadan
                            && date_response.month_number != 9
                        {
                            let mut args = HashMap::new();
                            args.insert("month", date_response.month_name);

                            let message =
                                i18n.t_with_args(&TranslationKey::WhiteDaysNotification, args);
                            BotCore::send_message(&bot, ChatId(chat_id), message).await;
                            return;
                        }

                        log::info!(
                            "Current Hijri date is {}, not scheduling white days message.",
                            date_response
                        );
                    }
                    Err(_err) => {
                        log::error!("Current Hijri date fetch error");
                    }
                }
            })
        })
        .map_err(|err| {
            log::error!("Failed to create white days message job: {}", err);
            AppErrorKind::WhiteDaysMessage
        })?;

        let job_data = white_days_message_job.job_data().map_err(|err| {
            log::error!("Failed to get job data: {}", err);
            AppErrorKind::WhiteDaysMessage
        })?;

        let extra_data = serde_json::to_vec(&JobExtraData {
            extension_type: JobExtensionType::WhiteDaysMessage,
        })
        .map_err(|err| {
            log::error!("Failed to serialize job extra data: {}", err);
            AppErrorKind::WhiteDaysMessage
        })?;

        white_days_message_job
            .set_job_data(JobStoredData {
                extra: extra_data,
                ..job_data
            })
            .map_err(|err| {
                log::error!("Failed to set job data: {}", err);
                AppErrorKind::WhiteDaysMessage
            })?;

        self.sched
            .add(white_days_message_job.clone())
            .await
            .map_err(|err| {
                log::error!("Failed to schedule white days message job: {}", err);
                AppErrorKind::WhiteDaysMessage
            })?;

        log::info!("White days message job scheduled successfully.");

        Ok(())
    }
}
