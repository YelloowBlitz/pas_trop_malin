use std::array::IntoIter;
use std::collections::HashMap;
use std::future::Future;
use std::io::Cursor;
use log::{info, LevelFilter, warn};
use regex::{Regex, Match};
use serenity::Client;
use serenity::model::channel::{Channel, ChannelCategory, GuildChannel, Message, PartialGuildChannel, Reaction, ReactionType, StageInstance};
use serenity::prelude::{EventHandler, Context};
use serenity::async_trait;
use serenity::http::Http;
use serenity::model::gateway::{Activity, Ready};
use serenity::model::prelude::{ChannelId, MessageId, User};
use serenity::model::prelude::Action::Emoji;
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
                index = i;
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
        // POLICE -> Vous etes en état d'arrestation
    //     let re = regex::Regex::new("di").unwrap();
    //     if let Some(m) = re.find(&*msg.content) {
    //         println!("Di !");
    //         let mut bot_user = ctx.cache.current_user().await;
    //
    //         let url = msg.author.avatar_url().unwrap();
    //         let img_bytes = reqwest::get(url).await.unwrap().bytes().await.unwrap();
    //         let image = image::load_from_memory(&img_bytes).unwrap();
    //         let mut buf = vec![];
    //         image.write_to(&mut Cursor::new(&mut buf), image::ImageOutputFormat::Png).unwrap();
    //         let res_base64 = base64::encode(buf);
    //
    //         let mut avatar = "data:image/png;base64,".to_owned();
    //         avatar.push_str(&res_base64);
    //
    //         bot_user.edit(&ctx.http, |p| p.avatar(Some(&avatar))).await.unwrap();
    //         info!("{}", &msg.author.avatar.unwrap());
    //
    //         if let Err(why) = msg.channel_id.say(&ctx.http, &msg.content[m.end()..]).await {
    //             warn!("Error sending message: {}", why);
    //         }
    //
    //         let avatar = serenity::utils::read_image("./avatar.png").unwrap();
    //         bot_user.edit(&ctx.http, |p| p.avatar(Some(&avatar))).await.unwrap();
    //     }
    }

    async fn ready(&self, ctx: Context, mut _data_about_bot: Ready) {
        ctx.set_activity(Activity::listening("N'importe quoi")).await;
        println!("Bot ready !");
    }
}

#[tokio::main]
async fn main() {
    // my_logger::set_logger(LevelFilter::Debug);

    // let token = "MzUwNjg5NDIzODc4Mzg5NzYx.WaBa-g.Hl5N9mX4qIzYV9KPdQf3uOStjN8";
    let token = "OTYwODgyNzM1MTMwNDg4OTAy.Ykw5yA.lLAQmK-Hy6pJk5oS-w3sIlYTcq8";

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
