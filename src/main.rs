use std::sync::LazyLock;
use std::{env, fs};
mod commands;
mod handler;
mod read_conf;

use commands::{initrolechannel, modmail, modmail_admin, register};
use dotenv::dotenv;
use handler::Handler;
use indexmap::IndexMap;
use poise::serenity_prelude as serenity;
use read_conf::{ModMailConfig, RoleConfig};

use serenity::model::prelude::*;

pub struct Data {}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

static MOD_MAIL_CONFIG: LazyLock<ModMailConfig> = LazyLock::new(|| {
    ModMailConfig::from_config(fs::read_to_string("modmail.toml").unwrap().as_str())
});

static ROLE_CONFIG: LazyLock<RoleConfig> =
    LazyLock::new(|| RoleConfig::from_config(fs::read_to_string("roles.toml").unwrap().as_str()));

static ROLE_MAP: LazyLock<IndexMap<String, RoleId>> = LazyLock::new(|| {
    IndexMap::from_iter(
        (ROLE_CONFIG)
            .choices
            .iter()
            .map(|c| &(c.options))
            .flat_map(|c| {
                c.iter()
                    .map(|(b_id, b)| (b_id.clone(), b.role_id))
                    .collect::<Vec<(String, RoleId)>>()
            }),
    )
});

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = serenity::GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![modmail(), initrolechannel(), register(), modmail_admin()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let mut client = serenity::Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
