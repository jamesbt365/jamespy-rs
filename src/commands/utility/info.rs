use crate::{Context, Error};
use poise::serenity_prelude::{self as serenity, Role, Permissions};

fn bool_converter(b: bool) -> String {
    if b {
        "Yes".to_string()
    } else {
        "No".to_string()
    }
}

/// Check the info about a specific role!
#[poise::command(
    rename = "role-info",
    aliases("role_info", "roleinfo"),
    slash_command,
    prefix_command,
    guild_only,
    category = "Utility",
    user_cooldown = "5"
)]
pub async fn role_info(
    ctx: Context<'_>,
    #[description = "Role"] role: Role,
    #[description = "Show all permissions?"] show_all_permissions: Option<bool>,
) -> Result<(), Error> {
    let role_id = role.id.get().to_string();
    let role_name = role.name;
    let colour = format!("#{}", role.colour.hex());
    let mention = format!("`<@&{}>`", role_id);
    let hoisted = bool_converter(role.hoist);
    let mentionable = bool_converter(role.mentionable);
    let managed = bool_converter(role.managed);
    let permissions = role.permissions;
    let permissions_title = if let Some(true) = show_all_permissions {
        "Permissions".to_string()
    } else {
        "Key Permissions".to_string()
    };

    let key_permissions = vec![
        Permissions::ADMINISTRATOR,
        Permissions::MANAGE_GUILD,
        Permissions::KICK_MEMBERS,
        Permissions::BAN_MEMBERS,
        Permissions::MANAGE_CHANNELS,
        Permissions::MANAGE_GUILD,
        Permissions::VIEW_AUDIT_LOG,
        Permissions::PRIORITY_SPEAKER,
        Permissions::SEND_TTS_MESSAGES,
        Permissions::MANAGE_MESSAGES,
        Permissions::MENTION_EVERYONE,
        Permissions::VIEW_GUILD_INSIGHTS,
        Permissions::MUTE_MEMBERS,
        Permissions::DEAFEN_MEMBERS,
        Permissions::MOVE_MEMBERS,
        Permissions::MANAGE_ROLES,
        Permissions::MANAGE_WEBHOOKS,
        Permissions::MANAGE_EMOJIS_AND_STICKERS,
        Permissions::MANAGE_THREADS,
        Permissions::CREATE_PRIVATE_THREADS,
        Permissions::MODERATE_MEMBERS, // Timeout
        Permissions::VIEW_CREATOR_MONETIZATION_ANALYTICS,
    ];

    let formatted_permissions: Vec<String> = permissions.iter().filter_map(|permission| {
        if show_all_permissions.unwrap_or(false) || key_permissions.contains(&permission) {
            Some(if key_permissions.contains(&permission) {
                // Highlight key permissions
                format!("**{}**", permission)
            } else {
                format!("{}", permission)
            })
        } else {
            None
        }
    }).collect();

    let permissions_list = formatted_permissions.join(", ");

    let embed = serenity::CreateEmbed::default()
        .field("ID", role_id, true)
        .field("Name", role_name, true)
        .field("Colour", colour, true)
        .field("Mention", mention, true)
        .field("Hoisted?", hoisted, true)
        .field("Mentionable?", mentionable, true)
        .field("Managed?", managed, true)
        .field(permissions_title, permissions_list, false);

    // In the future, maybe add a footer with the creation date, but I need to do math off the timestamp.

    let message = poise::CreateReply::default().embed(embed);
    ctx.send(message).await?;
    Ok(())
}


