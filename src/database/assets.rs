#![cfg(feature = "assets")]
//! Handles loading and scraping of external assets into the database.

use std::{
    collections::{BTreeSet, HashMap},
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    str::FromStr
};
use displaydoc::Display;
use itertools::Itertools;
use regex_lite::Regex;
use thiserror::Error;
use crate::database::Database;

use super::structures::{Color, TileData, Tiling};

// Taken from once_cell docs
macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| Regex::new($re).unwrap())
    }};
}

#[derive(Debug, Display, Error)]
/// Error when loading assets
pub enum LoadError {
    #[displaydoc("IO error: {0}")]
    /// Error opening a file
    IoError(#[from] io::Error),
    #[displaydoc("Decoding error: {0}")]
    /// Error when decoding a TOML file
    TomlError(#[from] toml::de::Error),
    #[displaydoc("Couldn't find data object {0} in lua file {1}")]
    /// Couldn't find data object in lua file
    LuaDataNotFound(&'static str, PathBuf),
    /// A lua file was invalid: {0}
    InvalidLua(&'static str),
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

    /// Parse a 2-element numeric tuple from Lua.
    fn parse_lua_vec2(col: &&str) -> Option<(u8, u8)> {
        col // Strip braces
            .strip_prefix('{')
            .and_then(|col| col.strip_suffix('}'))
            // Split by comma
            .and_then(|col| col.split_once(','))
            // Parse numbers
            .and_then(|(x, y)| Some((
                u8::from_str(x.trim()).ok()?,
                u8::from_str(y.trim()).ok()?
            )))
    }

    /// Parse a string vector from Lua.
    fn parse_lua_strings(col: &&str) -> Option<BTreeSet<String>> {
        col // Strip braces
            .strip_prefix('{')
            .and_then(|col| col.strip_suffix('}'))
            // Split by comma
            .map(|col| col.split(','))
            // Strip quotes
            .map(|strings| {
                strings.map(|string| 
                    string.trim().trim_matches('"').to_string()
                ).collect()
            })
    }

    /// Loads assets from `values.lua`.
    fn load_vanilla_values(&mut self, path: impl AsRef<Path>) -> Result<(), LoadError> {
        // Read the file
        let path = path.as_ref();
        let mut file_buf = String::new();
        let mut f = fs::File::open(path)?;
        f.read_to_string(&mut file_buf)?;
        drop(f); // Close immediately

        // Find the start and end of the tiles list
        let start = file_buf.find("tileslist =\n{\n\t")
            .ok_or(LoadError::LuaDataNotFound("tileslist", path.to_path_buf()))?;
        let len = file_buf[start..].find("\n}")
            .ok_or(LoadError::LuaDataNotFound("tileslist", path.to_path_buf()))?;
        // Slice the string to the tiles list
        // We add 15 to get to the end of start's pattern string,
        // and 2 to get past the end of the last object
        let tileslist_string = &file_buf[start + 13 .. start + len];
        let tiles = regex!(r"(?s-u)(\w+) =\n\t\{\s+([[:ascii:]]+?\n)\t\},")
            .captures_iter(tileslist_string)
            .map(|c| c.extract())
            .map(|(_, [object_id, tile_string])| {
                // Match for the properties of the tile
                let tile_props: HashMap<&str, &str> =
                    regex!(r"(\w+) = (.+?),\n")
                        .captures_iter(tile_string)
                        .map(|c| c.extract())
                        .map(|(_, [key, value])| (key, value))
                        .collect();
                (Some(object_id.trim_matches('"')), tile_props)
            })
            // Filter out the nonexistent tiles
            .filter(|(_, props)| props.get("does_not_exist").is_none())
            // Parse the id and props to a TileData
            .map(|(id, props)| Database::parse_data_from_strings(id, &props))
            .collect::<Result<HashMap<String, TileData>, LoadError>>()?;
        // It'd be really nice if we didn't have to collect in the middle,
        // but sadly, the error needs to propagate somehow.
        self.tiles.extend(tiles);
        Ok(())
    }

    /// Parses a tile's data from strings.
    fn parse_data_from_strings(object_id: Option<&str>, props: &HashMap<&str, &str>) -> Result<(String, TileData), LoadError> {
        // Parse name
        let name = (
            *props.get("name")
            .ok_or(LoadError::InvalidLua("no name"))?
        ).trim_matches('"').to_string();        // Parse color
        // Parse color
        let (color_x, color_y) = props.get("colour_active")
            .or(props.get("colour"))
            .and_then(Database::parse_lua_vec2)
            .ok_or(LoadError::InvalidLua("no color"))?;
        let color = Color::Paletted {x: color_x, y: color_y};
        // Parse tiling
        let tiling_num = props.get("tiling")
            .copied()
            .map(i8::from_str)
            .transpose().ok().flatten().ok_or(LoadError::InvalidLua("invalid tiling"))?;
        let tiling = Tiling::try_from(tiling_num)
            .map_err(|_| LoadError::InvalidLua("no tiling"))?;
        // Parse author
        let author = (
            *props.get("author")
            .unwrap_or(&"Hempuli")
        ).trim_matches('"').to_string();
        // Parse sprite
        let sprite = (
            *props.get("sprite").or(props.get("name"))
            .ok_or(LoadError::InvalidLua("no name"))?
        ).trim_matches('"').to_string();
        // Parse tile index, if it's there
        let tile_index = props.get("tile")
            .map(Database::parse_lua_vec2)
            .map(|opt| opt.ok_or(LoadError::InvalidLua("invalid tile index")))
            .transpose()?;
        // Parse grid index, if it's there
        let grid_index = props.get("grid")
            .map(Database::parse_lua_vec2)
            .map(|opt| opt.ok_or(LoadError::InvalidLua("invalid grid index")))
            .transpose()?;
        // Parse the layer, if it's there
        let layer = props.get("layer")
            .copied()
            .map(u8::from_str)
            .transpose().map_err(|_| LoadError::InvalidLua("invalid layer number"))?;
        // Parse the tags, if they're there
        let tags = props.get("tags")
            .map(Database::parse_lua_strings)
            .map(|opt| opt.ok_or(LoadError::InvalidLua("invalid tag list")))
            .transpose()?
            // If there's no tags, make it the empty vector
            .unwrap_or_default();
        // Construct it (finally)
        Ok((name, TileData {
            color,
            sprite,
            directory: "vanilla".to_string(),
            tiling,
            author,
            tile_index,
            grid_index,
            object_id: object_id.map(String::from),
            layer,
            tags
        }))
    }

    /// Loads assets from `editor_objlist.lua`.
    fn load_vanilla_objlist(&mut self, path: impl AsRef<Path>) -> Result<(), LoadError> {
        // Read the file
        let path = path.as_ref();
        let mut file_buf = String::new();
        let mut f = fs::File::open(path)?;
        f.read_to_string(&mut file_buf)?;
        drop(f); // Close immediately

        // Find the start and end of the tiles list
        // Offset the start by the match string's length
        let start = file_buf.find("editor_objlist = {\n\t")
            .ok_or(LoadError::LuaDataNotFound("editor_objlist", path.to_path_buf()))? + 20;
        let len = file_buf[start..].find("\n}")
            .ok_or(LoadError::LuaDataNotFound("editor_objlist", path.to_path_buf()))?;

        // Slice the string to the object list
        let objlist_string = &file_buf[start .. start+len];
        let tiles = regex!(r"(?s)\[\d+?] = \{(.+?)\t\},")
            .captures_iter(objlist_string)
            .map(|c| c.extract())
            .map(|(_, [tile_string])| {
                // Match for the properties of the tile
                let tile_props: HashMap<&str, &str> =
                    regex!(r"(\w+) = (.+?),\n")
                        .captures_iter(tile_string)
                        .map(|c| c.extract())
                        .map(|(_, [key, value])| (key, value))
                        .collect();
                (None, tile_props)
            })
            // Filter out the nonexistent tiles
            .filter(|(_, props)| props.get("does_not_exist").is_none())
            // Parse the id and props to a TileData
            .map(|(id, props)| Database::parse_data_from_strings(id, &props))
            .collect::<Result<HashMap<String, TileData>, LoadError>>()?;
        // It'd be really nice if we didn't have to collect in the middle,
        // but sadly, the error needs to propagate somehow.
        for (name, data) in tiles {
            let entry = self.tiles.entry(name).or_default();
            *entry = TileData {
                color: data.color,
                sprite: data.sprite,
                directory: data.directory,
                tiling: data.tiling,
                author: data.author,
                tile_index: data.tile_index.or(entry.tile_index),
                grid_index: data.grid_index.or(entry.grid_index),
                object_id: data.object_id.or(entry.object_id.clone()),
                layer: data.layer.or(entry.layer),
                tags: data.tags.union(&entry.tags).cloned().collect(),
            };
        }
        Ok(())
    }
}
