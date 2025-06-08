use std::sync::LazyLock;
use std::{env, fs};
pub mod read_conf;

use dotenv::dotenv;
use indexmap::IndexMap;
use read_conf::{PurgeTimerConfig, RoleButton, RoleConfig};
use serenity::all::CreateActionRow;
use serenity::async_trait;
use serenity::builder::{
    CreateButton, CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage,
};
use serenity::futures::StreamExt;
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

async fn delete_all_messages(ctx: &Context, channel_id: &ChannelId) {
    loop {
        let mut messages = channel_id.messages_iter(&ctx).boxed();
        let mut m_vec = vec![];
        while let Some(m_res) = messages.next().await {
            if let Ok(m) = m_res {
                m_vec.push(m);
            }
        }
        if m_vec.is_empty() {
            break;
        }
        channel_id.delete_messages(&ctx, m_vec).await.unwrap();
    }
}

impl Handler {
    async fn initialise_role_channel(&self, ctx: &Context, channel_id: &ChannelId) {
        delete_all_messages(ctx, channel_id).await;
        for choices in &*ROLE_CONFIG.choices {
            channel_id
                .send_message(
                    ctx,
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
}

#[async_trait]
impl EventHandler for Handler {
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

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        self.initialise_role_channel(&ctx, &ROLE_CONFIG.channel_id)
            .await;

        let _ = &*ROLE_MAP;

        tokio::spawn(async move {
            let config = fs::read_to_string("purge.toml").unwrap();
            let config = PurgeTimerConfig::from_config(config.as_str());
            let purge_time = config.time.time.unwrap();

            let now = chrono::Utc::now();
            let mut start = now
                .date_naive()
                .and_hms_opt(
                    purge_time.hour.into(),
                    purge_time.minute.into(),
                    purge_time.second.into(),
                )
                .unwrap()
                .signed_duration_since(now.naive_utc());
            let period = chrono::Duration::hours(24).to_std().unwrap();

            if start < chrono::Duration::zero() {
                start = start.checked_add(&chrono::Duration::hours(24)).unwrap();
            }

            let mut interval = tokio::time::interval_at(
                tokio::time::Instant::now() + start.to_std().unwrap(),
                period,
            );

            loop {
                interval.tick().await;

                delete_all_messages(&ctx, &config.channel_id).await;

                let next_purge = chrono::Utc::now()
                    .checked_add_days(chrono::Days::new(1))
                    .unwrap()
                    .timestamp();

                config
                    .channel_id
                    .send_message(
                        &ctx,
                        CreateMessage::new()
                            .content(format!("Channel will be purged in <t:{next_purge}:R>")),
                    )
                    .await
                    .unwrap();
            }
        });

        println!("{} has setup!", ready.user.name);
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
