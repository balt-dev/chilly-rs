#![cfg(feature = "assets")]
//! Handles loading of external assets into the database.

use std::collections::HashMap;
use std::io::Read;
use std::{fs, io};
use std::path::{Path, PathBuf};
use displaydoc::Display;
use itertools::Itertools;
use thiserror::Error;
use crate::database::Database;

use super::structures::TileData;

#[derive(Debug, Display, Error)]
/// Error when loading assets
pub enum LoadError {
    #[displaydoc("IO error: {0}")]
    /// Error opening a file
    IoError(#[from] io::Error),
    #[displaydoc("Decoding error: {0}")]
    /// Error when decoding a TOML file
    TomlError(#[from] toml::de::Error),
    /// Couldn't find data object in lua file
    LuaDataNotFound
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
        // Read the sprites file
        let dir_name = path.as_ref()
            .file_name()
            .ok_or(
                LoadError::IoError(io::Error::other("no world name found"))
            )?
            .to_string_lossy()
            .to_string();
        let sprites_path = path.as_ref().join("sprites.toml");
        let mut file_buf = String::new();
        let mut f = fs::File::open(sprites_path)?;
        f.read_to_string(&mut file_buf)?;
        drop(f); // Close immediately
        // Deserialize
        let data: HashMap<String, TileData> = toml::from_str(&file_buf)?;
        self.tiles.extend(
            // Set the world of the tile
            data.into_iter().update(
              |(_, t)| t.directory = dir_name.clone()
            )
        );
        Ok(())
    }

    /// Loads assets from a game directory.
    /// 
    /// # Errors
    /// Bails if the path can't be read, or there's an issue parsing.
    pub fn load_vanilla(&mut self, path: impl AsRef<Path>) -> Result<(), LoadError> {
        self.load_vanilla_values(path.as_ref().join("Data/values.lua"))?;
        self.load_vanilla_objlist(path.as_ref().join("Data/Editor/editor_objectlist.lua"))
    }

    /// Loads assets from `values.lua`.
    fn load_vanilla_values(&mut self, path: PathBuf) -> Result<(), LoadError> {
        // Read the file
        let mut file_buf = String::new();
        let mut f = fs::File::open(path)?;
        f.read_to_string(&mut file_buf)?;
        drop(f); // Close immediately

        // Find the start and end of the tiles list
        let start = file_buf.find("tileslist =\n{")
            .ok_or(LoadError::LuaDataNotFound)?;
        let end = file_buf[start..].find("}\n}")
            .ok_or(LoadError::LuaDataNotFound)?;
        // Slice the string to the tiles list
        // We add 13 to get to the end of start's pattern string
        let tileslist_string = &file_buf[start + 13 .. end];
        todo!()
    }

    /// Loads assets from `editor_objlist.lua`.
    fn load_vanilla_objlist(&mut self, path: PathBuf) -> Result<(), LoadError> {
        // Read the file
        let mut file_buf = String::new();
        let mut f = fs::File::open(path)?;
        f.read_to_string(&mut file_buf)?;
        drop(f); // Close immediately
        todo!()
    }
}
