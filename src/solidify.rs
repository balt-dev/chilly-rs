//! Holds methods and structures relating to solidifying a [`RawScene`] into a [`SkeletalScene`].


use std::collections::{BTreeMap, HashMap, HashSet};
use pest::Span;
use rand::seq::SliceRandom;

use crate::{
    arguments::{
        Flag, FlagName, TilingDirection, Variant
    }, database::{
        structures::{TileData, Tiling}, Database
    }, parser::{RawScene, TileTag}, structures::{Object, ObjectMap, Position}
};

/// The mode to default a tile to in a scene.
pub enum TileDefault {
    /// Default to regular tiles.
    Tile,
    /// Default to text objects.
    Text,
    /// Default to glyphs.
    Glyph
}

/// Helper macro for checking the neighbors of a tile
macro_rules! check_neighbors {
    (
        $acc: ident, $map: ident, $seed: ident, $conn_corners: ident;
        ($x: literal, $y: literal) => $var: ident,
        $(($xs: literal , $ys: literal) => $vars: ident),+
    ) => {
        check_neighbors!($acc, $map, $seed, $conn_corners; ($x, $y) => $var);
        check_neighbors!($acc, $map, $seed, $conn_corners; $(($xs, $ys) => $vars),+);
    };
    (
        $acc: ident, $map: ident, $seed: ident, $conn_corners: ident;
        ($x: literal, $y: literal) => $var: ident
    ) => {
        let mut __has_neighbor = false;
        if let Some(x) = $seed.x.checked_add_signed($x) {
            if let Some(y) = $seed.x.checked_add_signed($y) {
                let __pos = Position { x, y, ..$seed };
                if $map.contains_key(&__pos) {
                    __has_neighbor = true;
                }
            } else {
                __has_neighbor = $conn_corners;
            }
        } else {
            __has_neighbor = $conn_corners;
        }
        if __has_neighbor {
            $acc |= TileNeighbors::$var;
        }
    }
}

const ANIM_RIGHT: u8 = 0;
const ANIM_UP: u8 = 8;
const ANIM_LEFT: u8 = 16;
const ANIM_DOWN: u8 = 24;

impl<'scene> RawScene<'scene> {
    /// "Solidifies" the raw scene into a [`SkeletalScene`], applying any animation-level variants.
    ///
    /// # Panics
    /// Shouldn't panic, but does have `expect` in the body. If the code is broken, then it might.
    ///
    /// # Note
    /// There's an easter egg in Chilly coming from the community where
    /// a tile named `2` gets replaced with any character tile in the database
    /// and given a set of variants to alter its appearance.
    ///
    /// This can be disabled by leaving `easter_egg_tiles` empty.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn solidify<'db>(self, db: &'db Database, default: &TileDefault, easter_egg_tiles: &HashSet<String>) -> SkeletalScene<'db, 'scene> {
        let mut flags = self.flags;
        // Detect flags
        let connect_corners = flags.remove(&FlagName::ConnectBorders).is_some();
        let default_to_letters = flags.remove(&FlagName::UseLetters).is_some();
        // Parse scene
        let mut scene = SkeletalScene {
            letters: default_to_letters,
            flags,
            ..Default::default()
        };
        scene.map.height = self.map.height;
        scene.map.width = self.map.width;
        scene.map.length = self.map.length;
        // Construct the tiles
        let mut map = self.map;
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
        scene.map.objects = name_map.iter().map(|(pos, mut name)| {
            let pos = *pos;
            let tile = map.objects.remove(&pos).expect("positions from name map should sync with positions in tile map");
            // Apply animation frame-level variants
            let mut anim_frame: Option<(u8, u8)> = None;
            let new_variants = tile.variants.into_iter().filter(
                |variant| {
                    match variant {
                        Variant::AnimationFrame(frame)
                            => anim_frame = Some((*frame, *frame)),
                        Variant::Left()
                            => anim_frame = Some((ANIM_LEFT, ANIM_LEFT)),
                        Variant::Up()
                            => anim_frame = Some((ANIM_UP, ANIM_UP)),
                        Variant::Down()
                            => anim_frame = Some((ANIM_DOWN, ANIM_DOWN)),
                        Variant::Right()
                            => anim_frame = Some((ANIM_RIGHT, ANIM_RIGHT)),
                        Variant::Sleep() => {
                            let frame = anim_frame.unwrap_or_default().0;
                            // 0 gets wrapped to 255, which gets modulo'ed to 31, which is correct
                            let sleep_frame = frame.wrapping_sub(1) % 32;
                            anim_frame = Some((sleep_frame, frame));
                        },
                        Variant::Animation(cycle) => {
                            let mut frame = anim_frame.unwrap_or_default().0;
                            frame += cycle;
                            anim_frame = Some((frame, frame));
                        },
                        Variant::Tiling(directions) => {
                            let mut tiling = TileNeighbors::empty();
                            for direction in directions {
                                tiling |= match direction {
                                    TilingDirection::Up => TileNeighbors::UP,
                                    TilingDirection::Down => TileNeighbors::DOWN,
                                    TilingDirection::Left => TileNeighbors::LEFT,
                                    TilingDirection::Right => TileNeighbors::RIGHT,
                                    TilingDirection::UpRight => TileNeighbors::UPRIGHT,
                                    TilingDirection::UpLeft => TileNeighbors::UPLEFT,
                                    TilingDirection::DownLeft => TileNeighbors::DOWNLEFT,
                                    TilingDirection::DownRight => TileNeighbors::DOWNRIGHT
                                }
                            }
                            let frames = tiling.into_frames();
                            anim_frame = Some(frames);
                        },
                        _ => return true
                    }
                    false
                }
            ).collect();

            'handle_2: {
                if name == "2" {
                    // Easter egg! Grab any character tile currently in the database.
                    let characters = db.tiles.keys()
                        .filter(|name| easter_egg_tiles.contains(*name))
                        .collect::<Vec<_>>();
                    let Some(chosen_name) = characters
                        .choose(&mut rand::thread_rng())
                    else {
                        break 'handle_2
                    };
                    name = chosen_name;
                }
            }

            if let Some(data) = db.tiles.get(name) {
                if anim_frame.is_none() && data.tiling == Tiling::AutoTiled {
                    // Find the neighbors of this tile
                    let mut neighbors = TileNeighbors::empty();
                    check_neighbors! {
                        neighbors, name_map, pos, connect_corners;
                        ( 1,  0) => RIGHT,
                        ( 0, -1) => UP,
                        (-1,  0) => LEFT,
                        ( 0,  1) => DOWN,
                        ( 1, -1) => UPRIGHT,
                        (-1, -1) => UPLEFT,
                        (-1,  1) => DOWNLEFT,
                        ( 1,  1) => DOWNRIGHT
                    }
                    anim_frame = Some(neighbors.into_frames());
                }
                let anim_frame = anim_frame.unwrap_or_default();
                (pos, TileSkeleton {
                    data: TileSkeletonType::Existing(data),
                    animation_frame: anim_frame,
                    variants: new_variants,
                    span: tile.span
                })
            } else {
                (pos, TileSkeleton {
                    data: TileSkeletonType::Generative(name.clone()),
                    animation_frame: anim_frame.unwrap_or_default(),
                    variants: new_variants,
                    span: tile.span
                })
            }
        }).collect();
        scene
    }
}

/// The type of a [`TileSkeleton`].
#[derive(Debug, Clone, PartialEq)]
pub enum TileSkeletonType<'db> {
    /// Backed by database data
    Existing(&'db TileData),
    /// Does not exist, may need to be generated
    Generative(String)
}

/// A single tile, after tile-level parsing efforts have been made.
#[derive(Debug, Clone, PartialEq)]
pub struct TileSkeleton<'db, 'scene> {
    /// The backing data of this tile.
    pub data: TileSkeletonType<'db>,
    /// The animation frame that this tile has, with a fallback if the sprite for it doesn't exist.
    pub animation_frame: (u8, u8),
    /// The variants that this tile has.
    pub variants: Vec<Variant>,
    /// The span of the tile's name. Used for error reporting.
    pub(crate) span: Span<'scene>
}

impl<'a, 'scene> Object for TileSkeleton<'a, 'scene> {}

/// A scene that has been parsed, but with no rendering efforts done yet.
#[derive(Debug, Clone, Default)]
pub struct SkeletalScene<'db, 'scene> {
    /// A tilemap of the objects in the scene.
    pub map: ObjectMap<TileSkeleton<'db, 'scene>, usize>,
    /// Whether any generated text objects default to letters.
    pub letters: bool,
    /// The attached flags of the scene.
    pub flags: HashMap<FlagName, Flag>
}


bitflags::bitflags! {
    #[derive(Copy, Clone, PartialEq, Eq)]
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
    ///
    /// Returns two animation frames, one to use as a fallback if the other doesn't exist.
    fn into_frames(mut self) -> (u8, u8) {
        if !self.contains(TileNeighbors::RIGHT & TileNeighbors::UP) {
            self.remove(TileNeighbors::UPRIGHT);
        }
        if !self.contains(TileNeighbors::RIGHT & TileNeighbors::DOWN) {
            self.remove(TileNeighbors::DOWNRIGHT);
        }
        if !self.contains(TileNeighbors::LEFT & TileNeighbors::DOWN) {
            self.remove(TileNeighbors::DOWNLEFT);
        }
        if !self.contains(TileNeighbors::LEFT & TileNeighbors::UP) {
            self.remove(TileNeighbors::UPLEFT);
        }
        let regular = self.into_frame()
            .expect("we handled the other cases above");
        let fallback = self.intersection(TileNeighbors::CARDINAL).into_frame()
            .expect("cardinal directions should always be valid");
        (regular, fallback)
    }

    /// Turns a tile neighbor bitfield into an animation frame.
    fn into_frame(self) -> Option<u8> {
        Some( match self.bits() {
            // Straightforward so far, just a bitfield
            0b0000_0000 => 0,
            0b1000_0000 => 1,
            0b0100_0000 => 2,
            0b1100_0000 => 3,
            0b0010_0000 => 4,
            0b1010_0000 => 5,
            0b0110_0000 => 6,
            0b1110_0000 => 7,
            0b0001_0000 => 8,
            0b1001_0000 => 9,
            0b0101_0000 => 10,
            0b1101_0000 => 11,
            0b0011_0000 => 12,
            0b1011_0000 => 13,
            0b0111_0000 => 14,
            0b1111_0000 => 15,
            // Messy from here on, requires hardcoding
            0b1100_1000 => 16,
            0b1110_1000 => 17,
            0b1101_1000 => 18,
            0b1111_1000 => 19,
            0b0110_0100 => 20,
            0b1110_0100 => 21,
            0b0111_0100 => 22,
            0b1111_0100 => 23,
            0b1110_1100 => 24,
            0b1111_1100 => 25,
            0b0011_0010 => 26,
            0b1011_0010 => 27,
            0b0111_0010 => 28,
            0b1111_0010 => 29,
            0b1111_1010 => 30,
            0b0111_0110 => 31,
            0b1111_0110 => 32,
            0b1111_1110 => 33,
            0b1001_0001 => 34,
            0b1101_0001 => 35,
            0b1011_0001 => 36,
            0b1111_0001 => 37,
            0b1101_1001 => 38,
            0b1111_1001 => 39,
            0b1111_0101 => 40,
            0b1111_1101 => 41,
            0b1011_0011 => 42,
            0b1111_0011 => 43,
            0b1111_1011 => 44,
            0b1111_0111 => 45,
            0b1111_1111 => 46,
            _ => return None
        } )
    }
}
