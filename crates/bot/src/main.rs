#![warn(missing_docs, clippy::pedantic, clippy::perf)]
#![doc = include_str!("../README.md")]

use std::collections::HashMap;
use anyhow::anyhow;
use dirs::config_dir;
use shuttle_secrets::SecretStore;
use shuttle_service::error::Error;

mod bot;
mod handler;
mod hooks;
mod type_map;
mod structures;
mod groups;

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the config
    let config_path = config_dir().ok_or::<Error>(
        anyhow!("no config directory found for this OS").into()
    )?.join("chilly.toml");
    eprintln!("Reading config from {config_path:?}...");
    let token = secret_store.get("CHILL_TOKEN").ok_or::<Error>(
        anyhow!("CHILL_TOKEN was not found in secrets file").into()
    )?;
    let bot = bot::init(token, config_path).await.map_err(
        |err| anyhow!(err)
    )?;
    Ok(bot.into())
}
