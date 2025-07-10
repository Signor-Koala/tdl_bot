use std::sync::LazyLock;
use std::{env, fs};
pub mod read_conf;

use dotenv::dotenv;
use indexmap::IndexMap;
use read_conf::{RoleButton, RoleChoices, get_role_choices};
use serenity::all::CreateActionRow;
use serenity::async_trait;
use serenity::builder::{
    CreateButton, CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage,
};
use serenity::model::prelude::*;
use serenity::prelude::*;

fn create_role_button(choice_id: &str, choice_data: &RoleButton) -> CreateButton {
    CreateButton::new(choice_id)
        .emoji(
            choice_data
                .emoji
                .parse::<ReactionType>()
                .unwrap_or_else(|_| {
                    panic!("{} cannot be converted to an emoji", choice_data.emoji)
                }),
        )
        .label(choice_data.label.clone())
}

struct Handler;

static ROLE_CONFIG: LazyLock<Vec<RoleChoices>> =
    LazyLock::new(|| get_role_choices(fs::read_to_string("roles.toml").unwrap().as_str()));

static ROLE_MAP: LazyLock<IndexMap<String, RoleId>> = LazyLock::new(|| {
    IndexMap::from_iter((ROLE_CONFIG).iter().map(|c| &(c.options)).flat_map(|c| {
        c.iter()
            .map(|(b_id, b)| (b_id.clone(), b.role_id))
            .collect::<Vec<(String, RoleId)>>()
    }))
});

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
                                .map(|(i, d)| create_role_button(i, d))
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

        let sel_role = &interaction
            .guild_id
            .unwrap()
            .role(
                &ctx,
                *ROLE_MAP
                    .get(&interaction.data.custom_id)
                    .unwrap_or_else(|| panic!()),
            )
            .await
            .unwrap();

        let mem = interaction.member.clone().unwrap();

        let reply = if interaction
            .user
            .has_role(&ctx, &interaction.guild_id.unwrap(), sel_role)
            .await
            .unwrap()
        {
            mem.remove_role(&ctx, sel_role).await.unwrap();
            "You removed"
        } else {
            mem.add_role(&ctx, sel_role).await.unwrap();
            "You added"
        };

        interaction
            .create_response(
                &ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::default()
                        .ephemeral(true)
                        .content(format!("{} {}", reply, sel_role.name)),
                ),
            )
            .await
            .unwrap();
    }

    async fn ready(&self, _: Context, ready: Ready) {
        let _ = &*ROLE_CONFIG;
        let _ = &*ROLE_MAP;
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
