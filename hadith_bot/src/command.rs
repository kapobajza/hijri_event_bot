use teloxide::macros::BotCommands;

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    Help,
    Hadis,
    #[command(hide)]
    Start,
}
