use std::sync::Arc;

use bot_core::bot_core::BotCore;
use sqlx::{Pool, Postgres};
use teloxide::{
    prelude::Requester, repls::CommandReplExt, types::{BotName, Message}, ApiError, Bot, RequestError
};

use crate::{command::Command, db::HadithRepository, scheduler::Scheduler};

pub struct TelegramBot {
    hadith_repo: Arc<HadithRepository>,
    scheduler: Arc<Scheduler>,
    pool: Arc<Pool<Postgres>>,
    bot: Bot,
}

impl TelegramBot {
    pub fn new(pool: Pool<Postgres>, scheduler: Scheduler) -> Self {
        Self {
            hadith_repo: Arc::new(HadithRepository::new(pool.clone())),
            scheduler: Arc::new(scheduler),
            bot: Bot::from_env(),
            pool: Arc::new(pool),
        }
    }

    pub async fn run(&self) {
        log::info!("Starting Hadith bot...");

        let scheduler = Arc::clone(&self.scheduler);
        let bot = self.bot.clone();
        let hadith_repo = Arc::clone(&self.hadith_repo);
        let pool = Arc::clone(&self.pool);

        Command::repl(bot, move |bot: Bot, msg: Message, cmd: Command| {
            let scheduler = Arc::clone(&scheduler);
            let hadith_repo = Arc::clone(&hadith_repo);
            let pool = Arc::clone(&pool);

            async move {
                log::debug!("Received command: {:?}", cmd);

                match cmd {
                    Command::Help => {
                        BotCore::send_message(
                            &bot,
                            msg.chat.id,
                            "Dostupne komande:\n\n/help - Prikaži postojeće komande\n/hadis - Prikaži nasumični hadis".to_string(),
                        )
                        .await;
                    }
                    Command::Start => {
                        log::debug!("User started the bot: {:?}", msg.chat.id);
                        
                        let id = sqlx::query_scalar!(
                            "
                                INSERT INTO users (chat_id) 
                                VALUES ($1)
                                ON CONFLICT (chat_id)
                                DO UPDATE SET chat_id = EXCLUDED.chat_id
                                RETURNING id
                            ",
                            msg.chat.id.0 as i64,
                        )
                        .fetch_one(&*pool)
                        .await
                        .map_err(|e| {
                            log::error!("Failed to insert user: {}", e);
                            RequestError::Api(ApiError::CantInitiateConversation)
                        })?;
                            
                        match scheduler
                            .schedule_daily_hadith_job(bot.clone(), msg.chat.id, id)
                            .await
                        {
                            Ok(_) => {
                              let bot_name = bot.get_my_name().await.unwrap_or(BotName {
                                name: "Dnevni Hadis".to_string()
                              });

                              BotCore::send_message(
                                  &bot,
                                  msg.chat.id,
                                  format!("Es-selamu alejkum!\n\nJa sam {}. Svakog dana ću Vam slati jedan hadis.\nKoristite /help za listu komandi.", bot_name.name),
                              )
                              .await;
                            }
                            Err(err) => {
                                log::error!("Failed to schedule daily hadith job: {}", err);
                                BotCore::send_message(
                                    &bot,
                                    msg.chat.id,
                                    "Došlo je do greške prilikom zakazivanja dnevnog hadisa."
                                        .to_string(),
                                )
                                .await;
                            }
                        }
                    }
                    Command::Hadis => match hadith_repo.get_random_hadith_text().await {
                        Ok(hadith) => {
                            BotCore::send_message(&bot, msg.chat.id, hadith).await;
                        }
                        Err(e) => {
                            log::error!("Failed to fetch random hadith: {}", e);
                            BotCore::send_message(
                                &bot,
                                msg.chat.id,
                                "Dogodila se greška. Pokušajte kasnije.".to_string(),
                            )
                            .await;
                        }
                    },
                }

                Ok(())
            }
        })
        .await;
    }
}
