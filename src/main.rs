#![warn(clippy::pedantic)]
#![allow(clippy::unreadable_literal)]

mod data;
mod error;

use poise::serenity_prelude::{self as serenity};
use std::{sync::Arc, time::Duration};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().unwrap();

    let options = poise::FrameworkOptions {
        commands: moth_commands::commands(),
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("-".into()),
            edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                Duration::from_secs(600),
            ))),
            ..Default::default()
        },

        on_error: |error| Box::pin(error::handler(error)),

        command_check: Some(|ctx| Box::pin(moth_commands::command_check(ctx))),

        skip_checks_for_owners: false,
        event_handler: |framework, event| Box::pin(moth_events::event_handler(framework, event)),
        ..Default::default()
    };

    let framework = poise::Framework::new(options);

    let token = serenity::Token::from_env("MOTH_TOKEN")
        .expect("Missing `MOTH_TOKEN` environment variable.");
    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::GUILD_PRESENCES;

    let mut settings = serenity::Settings::default();
    settings.max_messages = 1000;

    let data = data::setup().await;

    let mut client = serenity::Client::builder(token, intents)
        .framework(framework)
        .data(data)
        .cache_settings(settings)
        .await
        .unwrap();

    client.start().await.unwrap();
}
