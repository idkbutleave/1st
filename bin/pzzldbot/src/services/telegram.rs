/*
    Appellation: telegram <module>
    Contrib: FL03 <j3mccain@gmail.com> (https://github.com/FL03)
    Description: ... Summary ...
*/
use scsys::AsyncResult;
use serde::{Deserialize, Serialize};
use teloxide::prelude::*;
use teloxide::dispatching::repls::CommandReplExt;
use teloxide::utils::command::BotCommands;


const DEFAULT_ENV_KEY: &str = "TELOXIDE_TOKEN";

trait TelegramBotSpec {
    fn name(&self) -> String where Self: Sized;
    fn username(&self) -> String where Self: Sized;
    fn bot_from_env() -> Bot where Self: Sized {
       Bot::from_env()
    }
    fn bot_with_token(token: String) -> Bot where Self: Sized {
        Bot::new(token)
    }
}

#[async_trait::async_trait]
trait TelegramBotOperator {
    fn client(&self) -> Bot where Self: Sized;
    async fn spawn(&self) -> AsyncResult where Self: Sized {
        Command::repl(self.client(), handler).await;
        Ok(())
    }
}


/// Defines the desired command structure for the bot
#[derive(BotCommands, Clone, Debug, PartialEq)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "Rolls a 6-sided die")]
    Dice,
    #[command(description = "display this text.")]
    Help,
    #[command(description = "Given a topic or url, return a concise summary")]
    Query(String),
}

/// Handles the commands issued to the bot and returns a [ResponseResult]
async fn handler(bot: Bot, cmd: Command, msg: Message) -> ResponseResult<()> {
    match cmd {
        Command::Dice => {
            bot.send_dice(msg.chat.id).await?;
        }
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
        }
        Command::Query(prompt) => {
            let oai = super::OpenAI::from_env(None);
            let req = oai.create_request(prompt.as_str());
            let res = oai.response(req).await.expect("");
            let choices = res.choices.iter().map(|i| format!("{:?}", i.text.clone())).collect::<String>();
            bot.send_message(msg.chat.id, choices).await?;
        },
    };

    Ok(())
}

#[derive(Clone, Debug, Default, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub struct TelegramBotToken(String);

impl TelegramBotToken {
    pub fn new(token: Option<String>) -> Self {
        Self(token.unwrap_or_default())
    }
    pub fn try_from_env(&mut self, token: Option<&str>) -> AsyncResult<&Self> {
        self.0 = std::env::var(token.unwrap_or(DEFAULT_ENV_KEY))?;
        Ok(self)
    }
}

impl TryFrom<Option<&str>> for TelegramBotToken {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn try_from(value: Option<&str>) -> Result<Self, Self::Error> {
        let res = std::env::var(value.unwrap_or(DEFAULT_ENV_KEY))?;
        Ok(Self::new(Some(res)))
    }
}

#[derive(Clone, Debug, Default, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub struct TelegramBotConfig {
    pub name: String,
    token: String,
    pub username: String,
}

impl TelegramBotConfig {
    pub fn new(name: String, token: String, username: String) -> Self {
        Self { name, token, username }
    }
    pub fn from_env(token: Option<&str>) -> Self {
        let token = std::env::var(token.unwrap_or(DEFAULT_ENV_KEY)).ok().unwrap();
        Self::new(Default::default(), token, Default::default())
    }
    pub fn try_from_env(token: Option<&str>) -> AsyncResult<Self> {
        let token = std::env::var(token.unwrap_or(DEFAULT_ENV_KEY))?;
        Ok(Self::new(Default::default(), token, Default::default()))
    }
}

#[derive(Clone, Debug, Default, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub struct TelegramBot {
    pub cnf: TelegramBotConfig,
}

impl TelegramBot {
    pub fn new(cnf: TelegramBotConfig) -> Self {
        Self { cnf }
    }
    pub fn bot(&self) -> Bot {
        Bot::new(self.cnf.token.clone())
    }
    pub async fn spawn(&self) -> AsyncResult {
        Command::repl(self.bot(), handler).await;
        Ok(())
    }
}

impl TelegramBotSpec for TelegramBotConfig {
    fn name(&self) -> String where Self: Sized {
        self.name.clone()
    }

    fn username(&self) -> String where Self: Sized {
        self.username.clone()
    }
}


impl TelegramBotSpec for TelegramBot {
    fn name(&self) -> String where Self: Sized {
        self.cnf.name()
    }

    fn username(&self) -> String where Self: Sized {
        self.cnf.username()
    }
}

impl TelegramBotOperator for TelegramBot {
    fn client(&self) -> Bot where Self:Sized {
        Bot::new(self.cnf.token.clone())
    }
}
