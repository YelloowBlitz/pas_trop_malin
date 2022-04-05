use std::sync::RwLock;
use regex::Regex;
use serenity::Client;
use serenity::model::channel::{Message, ReactionType};
use serenity::prelude::{EventHandler, Context};
use serenity::async_trait;
use serenity::model::gateway::{Activity, Ready};
use serenity::model::id::GuildId;
use serenity::model::interactions::{Interaction, InteractionResponseType};
use serenity::model::prelude::InteractionApplicationCommandCallbackDataFlags;
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
            let result = MessageBuilder::new()
                .push("Vous êtes en état d'arrestation ")
                .mention(&msg.author)
                .push(" !")
                .build();
            return Some((result, m.start()));
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

    fn repond(msg: &Message) -> Option<(String, usize)> {
        let re = Regex::new(r"(?im)(repond|répond)").unwrap();

        if let Some(m) = re.find(&msg.content) {
            if let Ok(s) = msg.content[m.end()..].trim().parse() {
                return Some((s, m.start()));
            }
        }

        None
    }
}

struct Handler(RwLock<bool>);

impl Handler {
    async fn message_sent(&self, ctx: &Context, message: Message) {
        if Regexes::koi(&message) {
            if let Err(why) = message.react(&ctx.http, ReactionType::Unicode("💇".parse().unwrap())).await {
                println!("Error react : {}", why);
            }
        }

        let (mut msg, mut index, mut reference_message) = (None, message.content.len(), false);

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
            if message.author.id != ctx.cache.current_user().await.id && i <= index {
                msg = Some(m);
                index = i;
            }
        }

        if let Some((m, i)) = Regexes::repond(&message) {
            if i <= index {
                msg = Some(m);
                reference_message = true;
                // index = i;
            }
        }


        if let Some(m) = msg {
            if let Err(why) = message.channel_id.send_message(&ctx.http, |cm| {
                if reference_message {
                    cm.reference_message(&message);
                    cm.allowed_mentions(|am| {
                        am.replied_user(true);
                        am
                    });
                }

                cm.content(m);

                cm
            }).await {
                println!("Error sending message : {}", why);
            }
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, message: Message) {
        if *self.0.read().unwrap() {
            self.message_sent(&ctx, message).await;
        }
    }

    async fn ready(&self, ctx: Context, mut ready: Ready) {
        ctx.set_activity(Activity::listening("N'importe quoi")).await;
        println!("Creating commands");
        for guild in ready.guilds {
            let guild_id = guild.id();

            let commands = GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
                commands
                    .create_application_command(|command| {
                        command
                            .name("stop")
                            .description("Stop the bot from reading messages")
                    })
                    .create_application_command(|command| {
                        command
                            .name("play")
                            .description("Let the bot read messages (same as resume)")
                    })
                    .create_application_command(|command| {
                        command
                            .name("resume")
                            .description("Let the bot read messages (same as again)")
                    })
                }).await;

            if let Err(why) = commands {
                println!("Failed to create a command : {}", why);
            }
        }
        println!("Bot ready !");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = &interaction {
            if command.data.name.as_str() == "stop" {
                *self.0.write().unwrap() = false;
                if let Err(why) = command.create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|d| {
                            d
                                .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                                .content("Bot stopped")
                        })
                }).await {
                    println!("Error responding to /stop : {}", why);
                }
            }
            if command.data.name.as_str() == "play" || command.data.name.as_str() == "resume" {
                *self.0.write().unwrap() = true;
                if let Err(why) = command.create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|d| {
                            d
                                .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                                .content("Bot resumed")
                        })
                }).await {
                    println!("Error responding to /stop : {}", why);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // my_logger::set_logger(LevelFilter::Debug);

    let token = std::env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let application_id = std::env::var("APP_ID")
        .expect("Expected an application id in the environment").parse().expect("Application ID must be u64");

    let mut client = Client::builder(token)
        .event_handler(Handler(RwLock::new(true)))
        .application_id(application_id)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
