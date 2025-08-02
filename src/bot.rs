use std::{collections::HashMap, sync::Arc};

use sqlx::{Pool, Postgres, types::Uuid};
use teloxide::{
    ApiError, Bot, RequestError,
    prelude::Requester,
    repls::CommandReplExt,
    types::{ChatId, Message},
};

use crate::{api::HijriApi, command::Command, i18n::I18n, scheduler::Scheduler};

pub struct TelegramBot {
    api: Arc<HijriApi>,
    i18n: Arc<I18n>,
    bot: Bot,
    pool: Arc<Pool<Postgres>>,
    scheduler: Arc<Scheduler>,
}

impl TelegramBot {
    pub fn new(
        api: Arc<HijriApi>,
        i18n: Arc<I18n>,
        pool: Pool<Postgres>,
        scheduler: Scheduler,
    ) -> Self {
        Self {
            api,
            i18n,
            bot: Bot::from_env(),
            pool: Arc::new(pool),
            scheduler: Arc::new(scheduler),
        }
    }

    pub async fn send_message(bot: &Bot, chat_id: ChatId, text: String) {
        if let Err(err) = bot.send_message(chat_id, &text).await {
            error!("Failed to send message: {}", err);
        }
    }

    pub async fn run(&self) {
        log::info!("Starting Hijri bot...");

        let i18n = Arc::clone(&self.i18n);
        let api = Arc::clone(&self.api);
        let bot = self.bot.clone();
        let pool = self.pool.clone();
        let scheduler = Arc::clone(&self.scheduler);

        Command::repl(bot, {
            move |bot: Bot, msg: Message, cmd: Command| {
                let i18n = Arc::clone(&i18n);
                let api = Arc::clone(&api);
                let pool = Arc::clone(&pool);
                let scheduler = Arc::clone(&scheduler);

                async move {
                    log::debug!("Received command: {:?}", cmd);

                    match cmd {
                        Command::Help => {
                            TelegramBot::send_message(&bot, msg.chat.id, i18n.t("help")).await;
                        }
                        Command::Date => {
                            let res = api.get_current_hijri_date().await;

                            match res {
                                Err(e) => {
                                    log::error!("Error fetching current Hijri date: {}", e);

                                    TelegramBot::send_message(
                                        &bot,
                                        msg.chat.id,
                                        i18n.t(&e.message_translation_key),
                                    )
                                    .await;
                                }
                                Ok(response) => {
                                    let mut args = HashMap::new();

                                    args.insert("day", response.day);
                                    args.insert("month", response.month);
                                    args.insert("year", response.year);
                                    args.insert("month_name", response.month_name);
                                    args.insert("month_ar", response.month_ar);

                                    TelegramBot::send_message(
                                        &bot,
                                        msg.chat.id,
                                        i18n.t_with_args("current_hijri_date", args),
                                    )
                                    .await;
                                }
                            }
                        }
                        Command::Start => {
                            log::debug!("User started the bot: {:?}", msg.chat.id);
                            let user_id = Uuid::new_v4();
                            sqlx::query!(
                                "INSERT INTO users (id, chat_id, username) VALUES ($1, $2, $3) ON CONFLICT (chat_id) DO NOTHING",
                                user_id,
                                msg.chat.id.0 as i64,
                                msg.from.and_then(|m| m.username.clone())
                            )
                            .execute(&*pool).await.map_err(|e| {
                                log::error!("Failed to insert user: {}", e);
                                RequestError::Api(ApiError::CantInitiateConversation)
                            })?;
                            scheduler.schedule_white_days_message(bot.clone(), msg.chat.id.0, user_id).await;
                            TelegramBot::send_message(&bot, msg.chat.id, i18n.t("welcome_message"))
                                .await;
                        }
                    }

                    Ok(())
                }
            }
        })
        .await;
    }
}
