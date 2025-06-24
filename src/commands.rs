use serenity::all::{
    CreateButton, CreateChannel, CreateMessage, GuildId, PermissionOverwrite, Permissions,
};

use crate::{Context, Error, MOD_MAIL_CONFIG};

#[poise::command(prefix_command, slash_command)]
pub async fn modmail(
    ctx: Context<'_>,
    #[description = "Title"] message: Option<String>,
) -> Result<(), Error> {
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
                    allow: Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES,
                    deny: Permissions::CREATE_PUBLIC_THREADS | Permissions::CREATE_PRIVATE_THREADS,
                    kind: serenity::all::PermissionOverwriteType::Role(MOD_MAIL_CONFIG.mod_role),
                },
                PermissionOverwrite {
                    allow: Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES,
                    deny: Permissions::CREATE_PUBLIC_THREADS | Permissions::CREATE_PRIVATE_THREADS,
                    kind: serenity::all::PermissionOverwriteType::Member(ctx.author().id),
                },
            ];
            let chan = CreateChannel::new(format!("{}-{}", t, ctx.author().name))
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
