#![warn(clippy::pedantic)]
// clippy warns for u64 -> i64 conversions despite this being totally okay in this scenario.
#![allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::wildcard_imports,
    clippy::module_name_repetitions,
    clippy::too_many_lines,
    clippy::unreadable_literal,
    clippy::unused_async, // fix.
)]

use jamespy_data::structs::{Command, Context, Data, Error};

pub mod general;
pub mod meta;
pub mod owner;
pub mod register;
pub mod utility;

pub mod utils;

#[must_use]
pub fn commands() -> Vec<Command> {
    meta::commands()
        .into_iter()
        .chain(owner::commands())
        .chain(general::commands())
        .chain(utility::commands())
        .collect()
}

pub async fn command_check(ctx: Context<'_>) -> Result<bool, Error> {
    if ctx.author().bot() {
        return Ok(false);
    };

    if ctx.framework().options.owners.contains(&ctx.author().id) {
        return Ok(true);
    };

    let user_banned = {
        let data = ctx.data();
        let config = data.config.read().unwrap();

        if let Some(banned_users) = &config.banned_users {
            banned_users.contains(&ctx.author().id)
        } else {
            false
        }
    };

    if user_banned {
        notify_user_ban(ctx).await?;
        return Ok(false);
    }

    Ok(true)
}

async fn notify_user_ban(ctx: Context<'_>) -> Result<(), Error> {
    use poise::serenity_prelude as serenity;

    let user = ctx.author();
    let author = serenity::CreateEmbedAuthor::new(ctx.author().tag()).icon_url(user.face());

    let desc = "You have been banned from using the bot. You have either misused jamespy, wronged \
                the owner or done something else stupid.\n\nMaybe this will be reversed in the \
                future, but asking or bothering me for it won't make that happen :3";

    let embed = serenity::CreateEmbed::new()
        .author(author)
        .description(desc)
        .thumbnail(ctx.cache().current_user().face())
        .colour(serenity::Colour::RED);

    ctx.send(poise::CreateReply::new().embed(embed)).await?;
    Ok(())
}
