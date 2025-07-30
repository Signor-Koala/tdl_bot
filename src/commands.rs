use serenity::{
    all::{
        CreateActionRow, CreateButton, CreateChannel, CreateMessage, GuildChannel, GuildId,
        PermissionOverwrite, Permissions, ReactionType, User,
    },
    async_trait,
};
use songbird::events::{Event, EventContext, EventHandler as VoiceEventHandler};
use songbird::{TrackEvent, input::YoutubeDl};

use crate::{Context, Error, HttpKey, MOD_MAIL_CONFIG, ROLE_CONFIG, handler::delete_all_messages};

struct TrackErrorNotifier;

#[async_trait]
impl VoiceEventHandler for TrackErrorNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (state, handle) in *track_list {
                println!(
                    "Track {:?} encountered an error: {:?}",
                    handle.uuid(),
                    state.playing
                );
            }
        }
        None
    }
}

#[poise::command(slash_command)]
pub async fn join_vc(
    ctx: Context<'_>,
    #[description = "Channel to join"] chan: GuildChannel,
) -> Result<(), Error> {
    ctx.defer().await?;

    let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();
    let handler_lock = manager.join(ctx.guild_id().unwrap(), chan.id).await?;
    let mut handler = handler_lock.lock().await;
    handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);

    if !handler.is_deaf() {
        println!("Deafend");
        handler.deafen(true).await.unwrap();
    } else {
        println!(" not Deafend");
    }

    if handler.is_mute() {
        println!("unmuted");
        handler.mute(false).await.unwrap();
    } else {
        println!("jfadl");
    }
    ctx.reply("Successfully joined VC!").await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn leave_vc(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();
    if manager.get(guild_id).is_some() {
        if let Err(e) = manager.remove(guild_id).await {
            ctx.reply(format!("Failed: {e:?}")).await
        } else {
            ctx.reply("Left VC").await
        }
    } else {
        ctx.reply("Not in VC").await
    }?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn play_yt(
    ctx: Context<'_>,
    #[description = "Youtube URL"] url: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let guild_id = ctx.guild_id().unwrap();
    let http_client = {
        let data = ctx.serenity_context().data.read().await;
        data.get::<HttpKey>()
            .cloned()
            .expect("Guarenteed to exist in the typemap")
    };
    let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let src = YoutubeDl::new(http_client, url);

        let _ = handler.play_input(src.clone().into());
        ctx.reply("Playing song").await
    } else {
        ctx.reply("Not in a VC").await
    }?;

    Ok(())
}

#[poise::command(slash_command)]
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
                    .components(
                        choices
                            .options
                            .iter()
                            .collect::<Vec<_>>()
                            .chunks(5)
                            .map(|c| {
                                CreateActionRow::Buttons(
                                    c.iter()
                                        .map(|(i, d)| {
                                            CreateButton::new(i.to_string())
                                        .emoji(d.emoji.parse::<ReactionType>().unwrap_or_else(
                                            |_| {
                                                panic!(
                                                    "{} cannot be converted to an emoji",
                                                    d.emoji
                                                )
                                            },
                                        ))
                                        .label(d.label.clone())
                                        })
                                        .collect(),
                                )
                            })
                            .collect(),
                    ),
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
