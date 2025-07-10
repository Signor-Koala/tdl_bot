use std::sync::LazyLock;
use std::{env, fs};
pub mod read_conf;

use dotenv::dotenv;
use read_conf::{RoleChoices, get_role_choices};
use serenity::all::CreateActionRow;
use serenity::async_trait;
use serenity::builder::{
    CreateButton, CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage,
};
use serenity::model::prelude::*;
use serenity::prelude::*;

struct Handler;

static ROLE_CONFIG: LazyLock<Vec<RoleChoices>> =
    LazyLock::new(|| get_role_choices(fs::read_to_string("roles.toml").unwrap().as_str()));

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content != "!ping" {
            return;
        }
        for choices in &*ROLE_CONFIG {
            msg.channel_id
                .send_message(
                    &ctx,
                    CreateMessage::new()
                        .content(choices.message.clone())
                        .components(vec![CreateActionRow::Buttons(
                            choices
                                .options
                                .iter()
                                .map(|(choice_id, choice_data)| {
                                    CreateButton::new(choice_id.clone())
                                        .emoji(
                                            choice_data
                                                .emoji
                                                .parse::<ReactionType>()
                                                .unwrap_or_else(|_| {
                                                    panic!(
                                                        "{} cannot be converted to an emoji",
                                                        choice_data.emoji
                                                    )
                                                }),
                                        )
                                        .label(choice_data.label.clone())
                                })
                                .collect(),
                        )]),
                )
                .await
                .unwrap();
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let interaction = match interaction {
            Interaction::Component(i) => i,
            _ => unimplemented!(),
        };
        let place = &interaction.data.custom_id;

        interaction
            .create_response(
                &ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::default()
                        .ephemeral(true)
                        .content(format!("You chose {place}")),
                ),
            )
            .await
            .unwrap();
    }

    async fn ready(&self, _: Context, ready: Ready) {
        let _ = &*ROLE_CONFIG;
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
