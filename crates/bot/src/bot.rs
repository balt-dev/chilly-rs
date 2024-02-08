//! This module handles nearly everything with the discord bot.

use std::{
    collections::HashSet,
    fs::File,
    io::Read,
    path::Path,
    time::Instant
};
use serenity::all::{GatewayIntents, Http};
use serenity::Client;

use crate::structures::{Config, InitError};
use crate::handler;
use crate::hooks;
use crate::type_map::{BotUser, StartedTime};


/// Initializes the bot, without starting it.
///
/// # Errors
/// Will return an error if initialization fails. See [`InitError`].
pub async fn init(token: String, config_path: impl AsRef<Path>) -> Result<Client, InitError> {
    // Read the configuration
    let mut conf_file = File::open(config_path)?;
    let mut raw_config = String::new();
    conf_file.read_to_string(&mut raw_config)?;
    let config: Config = toml::from_str(&raw_config)?;

    let http = Http::new(&token);

    // Get the owners and user
    let (owners, bot_user) = {
        let info = http.get_current_application_info().await?;
        let mut owners = HashSet::new();
        if let Some(team) = info.team {
            owners.insert(team.owner_user_id);
        } else if let Some(owner) = &info.owner {
            owners.insert(owner.id);
        }
        let bot_user = http.get_current_user().await?;
        (owners, bot_user)
    };

    let framework = hooks::setup_framework(config, owners);
    // Only ask for what we need
    let intents =
        GatewayIntents::MESSAGE_CONTENT |
        GatewayIntents::GUILD_MESSAGE_REACTIONS |
        GatewayIntents::DIRECT_MESSAGE_REACTIONS |
        GatewayIntents::GUILD_MESSAGES |
        GatewayIntents::DIRECT_MESSAGES |
        GatewayIntents::GUILD_MESSAGE_TYPING |
        GatewayIntents::DIRECT_MESSAGE_TYPING;

    // Build the client
    let client = Client::builder(&token, intents)
        .event_handler(handler::Handler)
        .framework(framework)
        // Set up extra data
        .type_map_insert::<BotUser>(bot_user)
        .type_map_insert::<StartedTime>(Instant::now())
        // Finish
        .await?;

    Ok(client)
}
