use std::fs;

use ::serenity::{
    all::{
        ChannelId, CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage,
        EventHandler, GuildId, Interaction, Ready, RoleId,
    },
    async_trait,
    futures::StreamExt,
};
use poise::serenity_prelude as serenity;

use crate::{ROLE_MAP, read_conf::VerificationConfig};

use super::read_conf::PurgeTimerConfig;

pub struct Handler;

pub async fn delete_all_messages(ctx: &serenity::Context, channel_id: &ChannelId) {
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

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: serenity::Context, interaction: Interaction) {
        let interaction = match interaction {
            Interaction::Component(i) => i,
            _ => {
                eprintln!("Unimplemented interaction: {interaction:?}");
                return;
            }
        };

        if &interaction.data.custom_id == "modmail_button" {
            interaction.channel_id.delete(ctx).await.unwrap();
            return;
        }

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

    async fn ready(&self, ctx: serenity::Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        let _ = &*ROLE_MAP;
        let ctx2 = ctx.clone();

        tokio::spawn(async move {
            let config = VerificationConfig::from_config("verfication.toml");
            let verified_role_id = config.verification_role;
            let guild_id = config.guild_id;
            let verification_period: i64 =
                chrono::Duration::days(3).to_std().unwrap().as_secs() as i64;

            let now = chrono::Utc::now();
            let mut start = now
                .date_naive()
                .and_hms_opt(0, 0, 0)
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
                let now_timestamp = chrono::Utc::now().timestamp();
                let all_members = guild_id.members(&ctx2, None, None).await.unwrap();
                let unverified_members = all_members
                    .iter()
                    .filter(|m| !m.roles.contains(&verified_role_id));
                for m in unverified_members {
                    let join_time = m.joined_at;
                    if let Some(joined) = join_time
                        && now_timestamp - joined.timestamp() > verification_period
                    {
                        println!("{} has been verified", m.display_name());
                        m.add_roles(&ctx2, &[verified_role_id]).await.unwrap();
                    }
                }

                interval.tick().await;
            }
        });

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
