use std::{collections::HashMap, sync::Arc};

use bot_core::bot_core::BotCore;
use sqlx::{Pool, Postgres, types::Uuid};
use teloxide::{ApiError, Bot, RequestError, repls::CommandReplExt, types::Message};

use crate::{
    api::HijriApi,
    command::Command,
    i18n::{instance::I18n, translation_key::TranslationKey},
    scheduler::Scheduler,
};

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
                            BotCore::send_message(&bot, msg.chat.id, i18n.t(&TranslationKey::Help))
                                .await;
                        }
                        Command::Date => {
                            let res = api.get_current_hijri_date().await;

                            match res {
                                Err(_e) => {
                                    BotCore::send_message(
                                        &bot,
                                        msg.chat.id,
                                        i18n.t(&TranslationKey::ErrorCurrentDate),
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

                                    BotCore::send_message(
                                        &bot,
                                        msg.chat.id,
                                        i18n.t_with_args(&TranslationKey::CurrentHijriDate, args),
                                    )
                                    .await;
                                }
                            }
                        }
                        Command::Start => {
                            log::debug!("User started the bot: {:?}", msg.chat.id);
                            let user_id = Uuid::new_v4();
                            sqlx::query!(
                                "
                                    INSERT INTO users (id, chat_id, username) 
                                    VALUES ($1, $2, $3)
                                    ON CONFLICT (chat_id)
                                    DO UPDATE SET chat_id = EXCLUDED.chat_id
                                    RETURNING id
                                ",
                                user_id,
                                msg.chat.id.0 as i64,
                                msg.from.and_then(|m| m.username.clone())
                            )
                            .fetch_one(&*pool)
                            .await
                            .map_err(|e| {
                                log::error!("Failed to insert user: {}", e);
                                RequestError::Api(ApiError::CantInitiateConversation)
                            })?;
                            scheduler
                                .schedule_white_days_message(bot.clone(), msg.chat.id.0)
                                .await
                                .map_err(|_e| {
                                    RequestError::Api(ApiError::CantInitiateConversation)
                                })?;
                            BotCore::send_message(
                                &bot,
                                msg.chat.id,
                                i18n.t(&TranslationKey::WelcomeMessage),
                            )
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
