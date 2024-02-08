//! Hold data structures for use in the bot.

use std::env::VarError;
use std::path::PathBuf;
use displaydoc::Display;
use thiserror::Error;
use serde::Deserialize;

#[derive(Debug, Display, Error)]
/// Different things that can go wrong when initializing the bot.
pub enum InitError {
    /// bot token environment variable not set: {0}
    NoToken(#[from] VarError),
    /// serenity error: {0}
    SerenityError(#[from] serenity::Error),
    /// failed to read config file: {0}
    ConfigOpenFailed(#[from] std::io::Error),
    /// failed to deserialize config file: {0}
    ConfigDeserializeFailed(#[from] toml::de::Error)
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub prefixes: Vec<String>,
    pub baba_path: PathBuf
}
