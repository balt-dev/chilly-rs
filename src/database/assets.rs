#![cfg(feature = "assets")]
//! Handles loading of external assets into the database.

use std::{fs, io};
use std::path::Path;
use displaydoc::Display;
use itertools::Itertools;
use thiserror::Error;
use crate::database::Database;

#[derive(Debug, Display, Error)]
/// Error when loading assets
pub enum LoadError {
    #[displaydoc("IO error: {0}")]
    /// Error opening a file
    IoError(#[from] io::Error),
    #[displaydoc("Decoding error: {0}")]
    /// Error when decoding a TOML file
    TomlError(#[from] toml::de::Error)
}

impl Database {
    /// Load custom assets from a directory of worlds.
    ///
    /// # Errors
    /// Bails if the path can't be read, or there's an issue reading the tile data.
    pub fn load_custom(&mut self, path: impl AsRef<Path>) -> Result<(), LoadError> {
        fs::read_dir(path)?
            // Get the paths
            .map(|c| Ok(c?.path()))
            // Filter to only the directories
            .filter_ok(|path| path.is_dir())
            // Load each directory
            .try_for_each(|res| res.and_then(|path| self.load_custom_path(path)))
    }

    /// Loads custom assets from a single directory.
    ///
    /// # Errors
    /// Bails if the path can't be read, or there's an issue reading the tile data.
    fn load_custom_path(&mut self, path: impl AsRef<Path>) -> Result<(), LoadError> {
        todo!("{path}")
    }
}
