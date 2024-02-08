//! Hold data structures for use in the bot.

use std::env::VarError;
use std::path::PathBuf;
use displaydoc::Display;
use thiserror::Error;
use serde::Deserialize;

#[derive(Debug, Display, Error)]
/// Different things that can go wrong when initializing the bot.
pub enum InitError {
    #[displaydoc("bot token environment variable not set: {0}")]
    /// Bot token environment variable not set
    NoToken(#[from] VarError),
    #[displaydoc("serenity error: {0}")]
    /// Serenity error
    SerenityError(#[from] serenity::Error),
    #[displaydoc("failed to read config file: {0}")]
    /// Failed to read config file
    ConfigOpenFailed(#[from] std::io::Error),
    #[displaydoc("failed to deserialize config file: {0}")]
    /// Failed to deserialize config file
    ConfigDeserializeFailed(#[from] toml::de::Error)
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub prefixes: Vec<String>,
    pub baba_path: PathBuf
}
