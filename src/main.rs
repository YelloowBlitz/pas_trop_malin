use regex::Regex;
use serenity::Client;
use serenity::model::channel::{ChannelCategory, GuildChannel, Message, PartialGuildChannel, Reaction, ReactionType, StageInstance};
use serenity::prelude::{EventHandler, Context};
use serenity::async_trait;
use serenity::model::gateway::{Activity, Ready};
use serenity::utils::MessageBuilder;

struct Regexes;

impl Regexes {
    fn di(msg: &Message) -> Option<(String, usize)> {
        let re = Regex::new(r"(?im)(dis|dit|di)").unwrap();

        if let Some(m) = re.find(&msg.content) {
            if let Ok(s) = msg.content[m.end()..].trim().parse() {
                return Some((s, m.start()));
            }
        }

        None
    }

    fn cri(msg: &Message) -> Option<(String, usize)> {
        let re = Regex::new(r"(?im)(crie|cri)").unwrap();

        if let Some(m) = re.find(&msg.content) {
            if let Ok(s) = (msg.content[m.end()..].trim().to_uppercase() + " !").parse() {
                return Some((s, m.start()));
            }
        }

        None
    }

    fn koi(msg: &Message) -> bool {
        let re = Regex::new(r"(?im)(koi|quoi|qoi)").unwrap();

        re.is_match(&msg.content)
    }

    fn police(msg: &Message) -> Option<(String, usize)> {
        let re = Regex::new(r"(?im)police").unwrap();

        if let Some(m) = re.find(&msg.content) {
            return Some((String::from("Vous êtes en état d'arrestation !"), m.start()));
        }

        None
    }

    fn je_suis(msg: &Message) -> Option<(String, usize)> {
        let re = Regex::new(r"(?im)je\s*suis").unwrap();

        if let Some(m) = re.find(&msg.content) {
            let result = MessageBuilder::new()
                .push("Enchanté ")
                .push_bold_safe(&msg.content[m.end()..].trim())
                .push(", moi c'est pas trop malin")
                .build();
            return Some((result, m.start()));
        }

        None
    }
}

struct Handler;

impl Handler {
    async fn message_sent(&self, ctx: &Context, message: Message) {
        if Regexes::koi(&message) {
            if let Err(why) = message.react(&ctx.http, ReactionType::Unicode("💇".parse().unwrap())).await {
                println!("Error react : {}", why);
            }
        }

        let (mut msg, mut index) = (None, message.content.len());

        if let Some((m, i)) = Regexes::di(&message) {
            if i <= index {
                msg = Some(m);
                index = i;
            }
        }

        if let Some((m, i)) = Regexes::cri(&message) {
            if i <= index {
                msg = Some(m);
                index = i;
            }
        }

        if let Some((m, i)) = Regexes::police(&message) {
            if i <= index {
                msg = Some(m);
                index = i;
            }
        }

        if let Some((m, i)) = Regexes::je_suis(&message) {
            if i <= index {
                msg = Some(m);
                // index = i;
            }
        }


        if let Some(m) = msg {
            if let Err(why) = message.channel_id.say(&ctx.http, &m).await {
                println!("Error sending message : {}", why);
            }
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, message: Message) {
        self.message_sent(&ctx, message).await;
    }

    async fn ready(&self, ctx: Context, mut _data_about_bot: Ready) {
        ctx.set_activity(Activity::listening("N'importe quoi")).await;
        println!("Bot ready !");
    }
}

#[tokio::main]
async fn main() {
    // my_logger::set_logger(LevelFilter::Debug);

    let token = std::env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
