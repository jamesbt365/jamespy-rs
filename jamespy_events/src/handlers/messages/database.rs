use std::sync::LazyLock;

use chrono::Utc;
use regex::Regex;
use small_fixed_array::FixedString;
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo};
use sqlx::{query, Encode, Postgres, Transaction, Type};

use crate::Error;
use jamespy_data::database::EmoteUsageType;
use poise::serenity_prelude::Message;

use small_fixed_array::ValidLength;

static EMOJI_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<(a)?:([a-zA-Z0-9_]{2,32}):(\d{1,20})>").unwrap());

// Foreign trait foreign type stuff.
pub struct FuckRustRules<'a, LenT: ValidLength>(pub &'a FixedString<LenT>);

impl<LenT: ValidLength> std::ops::Deref for FuckRustRules<'_, LenT> {
    type Target = FixedString<LenT>;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<LenT: ValidLength> Type<Postgres> for FuckRustRules<'_, LenT> {
    fn type_info() -> PgTypeInfo {
        <&str as Type<Postgres>>::type_info()
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        <&str as Type<Postgres>>::compatible(ty)
    }
}

impl<LenT: ValidLength> Encode<'_, Postgres> for FuckRustRules<'_, LenT> {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> sqlx::encode::IsNull {
        <&str as Encode<Postgres>>::encode(self.as_str(), buf)
    }
}

pub(super) async fn insert_message(
    mut transaction: Transaction<'_, Postgres>,
    message: &Message,
) -> Result<(), Error> {
    let guild_id = message.guild_id.map(|g| g.get() as i64);
    let channel_id = message.channel_id.get() as i64;
    let user_id = message.author.id.get() as i64;
    let message_id = message.id.get() as i64;

    if let Some(guild_id) = guild_id {
        query!(
            "INSERT INTO guilds (guild_id)
             VALUES ($1)
             ON CONFLICT (guild_id) DO NOTHING",
            guild_id
        )
        .execute(&mut *transaction)
        .await?;
    }

    query!(
        "INSERT INTO channels (channel_id, guild_id)
         VALUES ($1, $2)
         ON CONFLICT (channel_id) DO NOTHING",
        channel_id,
        guild_id
    )
    .execute(&mut *transaction)
    .await?;

    query!(
        "INSERT INTO users (user_id)
         VALUES ($1)
         ON CONFLICT (user_id) DO NOTHING",
        user_id
    )
    .execute(&mut *transaction)
    .await?;

    query!(
        "INSERT INTO messages (message_id, guild_id, channel_id, user_id, content, created_at)
         VALUES ($1, $2, $3, $4, $5, $6)",
        message_id,
        guild_id,
        channel_id,
        user_id,
        &FuckRustRules(&message.content),
        message.id.created_at().unix_timestamp()
    )
    .execute(&mut *transaction)
    .await?;

    if !message.embeds.is_empty() {
        query!(
            "INSERT INTO embeds (message_id, embed_data)
             VALUES ($1, $2)
             ON CONFLICT (message_id) DO NOTHING",
            message_id,
            serde_json::to_value(message.embeds.clone())?
        )
        .execute(&mut *transaction)
        .await?;
    }

    for attachment in &message.attachments {
        query!(
            "INSERT INTO attachments (attachment_id, message_id, file_name, file_size, file_url)
             VALUES ($1, $2, $3, $4, $5)",
            attachment.id.get() as i64,
            message_id,
            &FuckRustRules(&attachment.filename),
            attachment.size as i32,
            &FuckRustRules(&attachment.url)
        )
        .execute(&mut *transaction)
        .await?;
    }

    for sticker in &message.sticker_items {
        let sticker_id = sticker.id.get() as i64;
        query!(
            "INSERT INTO stickers (sticker_id, sticker_name) VALUES ($1, $2) ON CONFLICT \
             (sticker_id) DO NOTHING",
            sticker_id,
            &FuckRustRules(&sticker.name)
        )
        .execute(&mut *transaction)
        .await?;

        query!(
            "INSERT INTO sticker_usage (message_id, user_id, channel_id, guild_id, sticker_id) \
             VALUES ($1, $2, $3, $4, $5)",
            message_id,
            user_id,
            channel_id,
            guild_id,
            sticker_id
        )
        .execute(&mut *transaction)
        .await?;
    }

    for captures in EMOJI_REGEX.captures_iter(&message.content) {
        let Ok(id) = &captures[3].parse::<u64>() else {
            println!("Failed to parse id for custom emote: {}", &captures[3]);
            continue;
        };
        // &captures[2] is name.
        // &captures[3] is id.
        query!(
            "INSERT INTO emotes (emote_name, discord_id) VALUES ($1, $2) ON CONFLICT (discord_id) \
             DO NOTHING",
            &captures[2],
            *id as i64
        )
        .execute(&mut *transaction)
        .await?;

        query!(
            "INSERT INTO emote_usage (message_id, emote_id, user_id, channel_id, guild_id,
             used_at, usage_type) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            message_id,
            *id as i64,
            user_id,
            channel_id,
            guild_id,
            message.id.created_at().unix_timestamp(),
            EmoteUsageType::Message as _,
        )
        .execute(&mut *transaction)
        .await?;
    }

    transaction.commit().await?;

    Ok(())
}

pub(super) async fn insert_edit(
    mut transaction: Transaction<'_, Postgres>,
    message: &Message,
) -> Result<(), Error> {
    let guild_id = message.guild_id.map(|g| g.get() as i64);
    let channel_id = message.channel_id.get() as i64;
    let user_id = message.author.id.get() as i64;
    let message_id = message.id.get() as i64;

    query!(
        "INSERT INTO channels (channel_id, guild_id)
         VALUES ($1, $2)
         ON CONFLICT (channel_id) DO NOTHING",
        channel_id,
        guild_id
    )
    .execute(&mut *transaction)
    .await?;

    if let Some(guild_id) = guild_id {
        query!(
            "INSERT INTO guilds (guild_id)
             VALUES ($1)
             ON CONFLICT (guild_id) DO NOTHING",
            guild_id
        )
        .execute(&mut *transaction)
        .await?;
    }

    query!(
        "INSERT INTO users (user_id)
         VALUES ($1)
         ON CONFLICT (user_id) DO NOTHING",
        user_id
    )
    .execute(&mut *transaction)
    .await?;

    let timestamp = message
        .edited_timestamp
        .map_or_else(|| Utc::now().timestamp(), |t| t.unix_timestamp());

    query!(
        "INSERT INTO message_edits (message_id, channel_id, guild_id, user_id, content, \
         edited_at) VALUES ($1, $2, $3, $4, $5, $6)",
        message_id,
        channel_id,
        guild_id,
        user_id,
        &FuckRustRules(&message.content),
        timestamp
    )
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(())
}

pub(super) async fn insert_deletion(
    mut transaction: Transaction<'_, Postgres>,
    message: &Message,
) -> Result<(), Error> {
    let guild_id = message.guild_id.map(|g| g.get() as i64);
    let channel_id = message.channel_id.get() as i64;
    let user_id = message.author.id.get() as i64;
    let message_id = message.id.get() as i64;

    query!(
        "INSERT INTO channels (channel_id, guild_id)
         VALUES ($1, $2)
         ON CONFLICT (channel_id) DO NOTHING",
        channel_id,
        guild_id
    )
    .execute(&mut *transaction)
    .await?;

    if let Some(guild_id) = guild_id {
        query!(
            "INSERT INTO guilds (guild_id)
             VALUES ($1)
             ON CONFLICT (guild_id) DO NOTHING",
            guild_id
        )
        .execute(&mut *transaction)
        .await?;
    }

    query!(
        "INSERT INTO users (user_id)
         VALUES ($1)
         ON CONFLICT (user_id) DO NOTHING",
        user_id
    )
    .execute(&mut *transaction)
    .await?;

    let timestamp = Utc::now().timestamp();

    query!(
        "INSERT INTO message_deletion (message_id, channel_id, guild_id, user_id, content, \
         deleted_at) VALUES ($1, $2, $3, $4, $5, $6)",
        message_id,
        channel_id,
        guild_id,
        user_id,
        &FuckRustRules(&message.content),
        timestamp
    )
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(())
}
