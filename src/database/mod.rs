//! Handles everything relating to data not included in the bot.

use std::collections::HashMap;
use crate::database::structures::TileData;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};


pub mod structures;
mod constants;
mod assets;

/// Chilly's internal database.
///
/// # Notes
/// This is expected to be a VERY large value, so cloning a filled database is unwise.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct Database {
    /// A mapping of tile names to their data.
    pub tiles: HashMap<String, TileData>
}

impl Database {
    /// Creates a new, empty database.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
