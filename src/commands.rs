use serenity::all::{
    CreateActionRow, CreateButton, CreateChannel, CreateMessage, GuildId, PermissionOverwrite,
    Permissions, ReactionType, User,
};

use crate::{Context, Error, MOD_MAIL_CONFIG, ROLE_CONFIG, handler::delete_all_messages};

#[poise::command(prefix_command, slash_command)]
pub async fn modmail(
    ctx: Context<'_>,
    #[description = "Title"] message: Option<String>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    match message {
        Some(t) => {
            let user_perm = [
                PermissionOverwrite {
                    allow: Permissions::empty(),
                    deny: Permissions::all(),
                    kind: serenity::all::PermissionOverwriteType::Role(GuildId::everyone_role(
                        &ctx.guild_id().unwrap(),
                    )),
                },
                PermissionOverwrite {
                    allow: Permissions::VIEW_CHANNEL
                        | Permissions::SEND_MESSAGES
                        | Permissions::READ_MESSAGE_HISTORY,
                    deny: Permissions::CREATE_PUBLIC_THREADS | Permissions::CREATE_PRIVATE_THREADS,
                    kind: serenity::all::PermissionOverwriteType::Role(MOD_MAIL_CONFIG.mod_role),
                },
                PermissionOverwrite {
                    allow: Permissions::VIEW_CHANNEL
                        | Permissions::SEND_MESSAGES
                        | Permissions::READ_MESSAGE_HISTORY,
                    deny: Permissions::CREATE_PUBLIC_THREADS | Permissions::CREATE_PRIVATE_THREADS,
                    kind: serenity::all::PermissionOverwriteType::Member(ctx.author().id),
                },
            ];
            let chan = CreateChannel::new(format!(
                "{}-{}",
                ctx.author().name,
                chrono::Utc::now().timestamp()
            ))
            .kind(serenity::all::ChannelType::Text)
            .category(MOD_MAIL_CONFIG.channel_id)
            .permissions(user_perm);
            let a = ctx.guild_id().unwrap().create_channel(&ctx, chan).await?;

            let m = CreateMessage::new()
                .content(format!("# Title: {t} \n Click here to delete the channel",))
                .button(CreateButton::new("modmail_button").label("Close Mod-Mail"));

            a.id.send_message(ctx, m).await?;
            ctx.reply(format!("Mod Mail Channel made at <#{}>", a.id))
                .await?;
        }
        None => {
            ctx.say("modmail must include a title!").await?;
        }
    };

    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "MANAGE_MESSAGES"
)]
pub async fn modmail_admin(
    ctx: Context<'_>,
    #[description = "Title"] message: Option<String>,
    #[description = "User"] suspect: Option<User>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    match message {
        Some(t) => {
            let suspect = suspect.unwrap().id;
            let user_perm = [
                PermissionOverwrite {
                    allow: Permissions::empty(),
                    deny: Permissions::all(),
                    kind: serenity::all::PermissionOverwriteType::Role(GuildId::everyone_role(
                        &ctx.guild_id().unwrap(),
                    )),
                },
                PermissionOverwrite {
                    allow: Permissions::VIEW_CHANNEL
                        | Permissions::SEND_MESSAGES
                        | Permissions::READ_MESSAGE_HISTORY,
                    deny: Permissions::CREATE_PUBLIC_THREADS | Permissions::CREATE_PRIVATE_THREADS,
                    kind: serenity::all::PermissionOverwriteType::Role(MOD_MAIL_CONFIG.mod_role),
                },
                PermissionOverwrite {
                    allow: Permissions::VIEW_CHANNEL
                        | Permissions::SEND_MESSAGES
                        | Permissions::READ_MESSAGE_HISTORY,
                    deny: Permissions::CREATE_PUBLIC_THREADS | Permissions::CREATE_PRIVATE_THREADS,
                    kind: serenity::all::PermissionOverwriteType::Member(suspect),
                },
            ];
            let chan = CreateChannel::new(format!("{t}-{suspect}"))
                .kind(serenity::all::ChannelType::Text)
                .category(MOD_MAIL_CONFIG.channel_id)
                .permissions(user_perm);
            let a = ctx.guild_id().unwrap().create_channel(&ctx, chan).await?;

            let m = CreateMessage::new()
                .content("Click here to delete the channel")
                .button(CreateButton::new("modmail_button").label("Close Mod-Mail"));

            a.id.send_message(ctx, m).await?;
            ctx.reply(format!("Mod Mail Channel made at <#{}>", a.id))
                .await?;
        }
        None => {
            ctx.say("modmail must include a title!").await?;
        }
    };

    Ok(())
}

#[poise::command(slash_command, default_member_permissions = "ADMINISTRATOR")]
pub async fn initrolechannel(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    delete_all_messages(ctx.serenity_context(), &ctx.channel_id()).await;
    for choices in &*ROLE_CONFIG.choices {
        ctx.channel_id()
            .send_message(
                ctx,
                CreateMessage::new()
                    .content(choices.message.clone())
                    .components(vec![CreateActionRow::Buttons(
                        choices
                            .options
                            .iter()
                            .map(|(i, d)| {
                                CreateButton::new(i)
                                    .emoji(d.emoji.parse::<ReactionType>().unwrap_or_else(|_| {
                                        panic!("{} cannot be converted to an emoji", d.emoji)
                                    }))
                                    .label(d.label.clone())
                            })
                            .collect(),
                    )]),
            )
            .await?;
    }
    ctx.reply("Role channel initialised").await?;
    Ok(())
}

#[poise::command(prefix_command, default_member_permissions = "ADMINISTRATOR")]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}
