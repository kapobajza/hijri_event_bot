use teloxide::{Bot, prelude::Requester, types::ChatId};

pub struct BotCore;

impl BotCore {
    pub async fn send_message(bot: &Bot, chat_id: ChatId, text: String) {
        if let Err(err) = bot.send_message(chat_id, &text).await {
            log::error!("Failed to send message: {}", err);
        }
    }
}
