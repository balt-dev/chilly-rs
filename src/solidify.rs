//! Holds methods and structures relating to solidifying a [`RawScene`] into a [`Scene`].


use std::collections::{BTreeMap, HashMap};

use crate::database::Database;
use crate::parser::{RawScene, TileTag};
use crate::structures::{Object, ObjectMap, Position};

/// The mode to default a tile to in a scene.
pub enum TileDefault {
    Tile,
    Text,
    Glyph
}

/// Helper macro for checking the neighbors of a tile
macro_rules! check_neighbors {
    ($acc: ident, $map: ident, $seed: ident; ($x: literal, $y: literal) => $var: ident , $(($xs: literal , $ys: literal) => $vars: ident),+) => {
        check_neighbors!($acc, $map, $seed; ($x, $y) => $var);
        check_neighbors!($acc, $map, $seed; $(($xs, $ys) => $vars),+)
    };
    ($acc: ident, $map: ident, $seed: ident; ($x: literal, $y: literal) => $var: ident) => {
        if let Some(x) = $seed.x.checked_add_signed($x) {
            if let Some(y) = $seed.x.checked_add_signed($y) {
                let __pos = Position { x, y, ..$seed };
                if $map.contains_key(&__pos) {
                    $acc |= TileNeighbors::$var;
                }
            }
        }
    }
} 

impl<'scene> RawScene<'scene> {
    /// Applies variants that affect raw tiles.
    pub fn apply_tile_variants<'db>(self, db: &'db Database, default: TileDefault) -> Scene<'db> {
        let mut flags = self.flags;
        let connect_corners = flags.contains_key("tb") || flags.contains_key("tile_borders");
        let default_to_letters = flags.contains_key("letter");
        // Remove flags we aren't using anymore
        flags.remove("tb");
        flags.remove("tile_borders");
        flags.remove("letter");
        let mut scene = Scene::default();
        // Set the properties of the scene
        scene.letters = default_to_letters;
        scene.flags = flags;
        scene.map.height = self.map.height;
        scene.map.width = self.map.width;
        scene.map.length = self.map.length;
        // Construct the tiles
        let map = self.map;
        let name_map = map.objects.iter().map(|(pos, tile)| {
            // Transform the name into its canonical representation
            let name = match (tile.tag, &default) {
                (Some(TileTag::Text), &TileDefault::Text) =>
                    tile.name.strip_prefix("text_").unwrap_or(tile.name).to_string(),
                (Some(TileTag::Text), _) | (None, &TileDefault::Text) =>
                    format!("text_{}", tile.name),
                (Some(TileTag::Glyph), &TileDefault::Glyph) =>
                    tile.name.strip_prefix("glyph_").unwrap_or(tile.name).to_string(),
                (Some(TileTag::Glyph), _) | (None, &TileDefault::Glyph) =>
                    format!("glyph_{}", tile.name),
                (None, &TileDefault::Tile) =>
                    tile.name.to_string()
            };
            (*pos, name)
        }).collect::<BTreeMap<_, _>>();
        let tiles = name_map.iter().map(|(pos, name)| {
            let pos = *pos;
            // Find the neighbors of each tile
            let mut neighbors = TileNeighbors::empty();
            check_neighbors! {
                neighbors, name_map, pos;
                ( 1,  0) => RIGHT,
                ( 0, -1) => UP,
                (-1,  0) => LEFT,
                ( 0,  1) => DOWN,
                ( 1, -1) => UPRIGHT,
                (-1, -1) => UPLEFT,
                (-1,  1) => DOWNLEFT,
                ( 1,  1) => DOWNRIGHT
            }
        })
        scene
    }
}

/// A single tile, after tile-level parsing efforts have been made.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TileSkeleton<'db> {
    /// A tile that has sprites in the database.
    ExistingTile {
        /// The sprite filename of this tile, if it has one.
        sprite: &'db str,
        directory: &'db str,
        variants: 
    },
    /// A tile that has no sprites in the database, and may need to have them generated.
    Generated {
        name: String
    }
}

impl<'a> Object for TileSkeleton<'a> {}

/// A scene, after parsing.
#[derive(Debug, Clone, Default)]
pub struct Scene<'db> {
    /// A tilemap of the objects in the scene.
    pub map: ObjectMap<TileSkeleton<'db>, usize>,
    /// Whether any generated text objects default to letters.
    pub letters: bool,
    /// The attached flags of the scene.
    pub flags: HashMap<String, Option<String>>
}


bitflags::bitflags! {
    struct TileNeighbors: u8 {
        const RIGHT =        0b1000_0000;
        const UP =           0b0100_0000;
        const LEFT =         0b0010_0000;
        const DOWN =         0b0001_0000;
        const UPRIGHT =      0b0000_1000;
        const UPLEFT =       0b0000_0100;
        const DOWNLEFT =     0b0000_0010;
        const DOWNRIGHT =    0b0000_0001;

        const CARDINAL = Self::UP.bits() | Self::LEFT.bits() | Self::DOWN.bits() | Self::RIGHT.bits();
    }
}

impl TileNeighbors {
    /// Maps the tile neighbors to an animation frame.
    /// If the configuration doesn't map to anything using
    pub fn to_frame(&self) -> (Option<u8>, u8) {
        let mut flags = *self;
        if !(self.contains(TileNeighbors::RIGHT) && self.contains(TileNeighbors::UP)) {
            flags.remove(TileNeighbors::UPRIGHT);
        }
    }
}