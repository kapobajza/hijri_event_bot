use std::{collections::HashMap, sync::Arc};

use sqlx::{Pool, Postgres, types::Uuid};
use teloxide::{Bot, types::ChatId};
use tokio_cron_scheduler::{Job, JobScheduler, SimpleJobCode, SimpleNotificationCode};

use crate::{
    api::HijriApi,
    bot::TelegramBot,
    db::{
        postgres_metadata_store::PostgresMetadataStore,
        postgres_notification_store::PostgresNotificationStore,
    },
    i18n::I18n,
};

pub struct Scheduler {
    pool: Arc<Pool<Postgres>>,
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
        let mut sched = JobScheduler::new_with_storage_and_code(
            Box::new(PostgresMetadataStore::new(pool.clone())),
            Box::new(PostgresNotificationStore::new(pool.clone())),
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

        Ok(Self {
            pool: Arc::new(pool),
            sched,
            api,
            i18n,
        })
    }

    pub async fn schedule_white_days_message(&self, bot: Bot, chat_id: i64, user_id: Uuid) {
        let pool = Arc::clone(&self.pool);
        let api = Arc::clone(&self.api);
        let i18n = Arc::clone(&self.i18n);
        let bot = Arc::new(bot);

        // This job will run at 18:00 every day to check if it's the 12th of the month
        // and send a notification if it is
        let white_days_message_job = Job::new_async("0 0 18 * * *", move |_uuid, _l| {
            let api = Arc::clone(&api);
            let i18n = Arc::clone(&i18n);
            let bot = Arc::clone(&bot);

            Box::pin(async move {
                let current_date = api.get_current_hijri_date().await;

                match current_date {
                    Ok(date_response) => {
                        if date_response.day_number == DAY_BEFORE_FIRST_WHITE_DAY {
                            let mut args = HashMap::new();
                            args.insert("month", date_response.month_name);

                            let message = i18n.t_with_args("white_days_notification", args);
                            TelegramBot::send_message(&bot, ChatId(chat_id), message).await;
                            return;
                        }

                        log::info!(
                            "Current Hijri date is {}, not scheduling white days message.",
                            date_response
                        );
                    }
                    Err(err) => {
                        log::error!("Failed to get current Hijri date: {}", err);
                    }
                }
            })
        });

        match white_days_message_job {
            Ok(job) => {
                if let Err(err) = self.sched.add(job.clone()).await {
                    let res = sqlx::query!(
                        "INSERT INTO job_extensions (job_id, user_id) VALUES ($1, $2)",
                        job.guid(),
                        user_id
                    )
                    .execute(&*pool)
                    .await;

                    if let Err(e) = res {
                        log::error!("Failed to insert job extension: {}", e);
                    }

                    log::error!("Failed to schedule white days message job: {}", err);
                } else {
                    log::info!("White days message job scheduled successfully.");
                }
            }
            Err(err) => {
                log::error!("Failed to create white days message job: {}", err);
            }
        }
    }
}
