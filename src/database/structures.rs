//! Data structures for use within the database.

use core::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use displaydoc::Display;

#[cfg(feature = "serde")]
use serde::{
    Serialize, Deserialize,
    Deserializer, Serializer,
    de::{self, SeqAccess},
    ser::SerializeSeq
};
#[cfg(feature = "serde")]
use serde_repr::{Serialize_repr, Deserialize_repr};

#[repr(i8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize_repr, Deserialize_repr))]
/// An enumeration over possible tiling modes for a tile.
pub enum Tiling {
    /// No tiling. Only has one sprite.
    #[default]
    None = -1,
    /// Has sprites for all four directions.
    Directional = 0,
    /// Has sprites for connecting to tiles of the same type.
    ///
    /// May or may not have special sprites for corner connections.
    AutoTiled = 1,
    /// Has sprites for a character-like tile.
    ///
    /// These have sprites for directions,
    /// animation frames within those directions,
    /// and an extra sleep frame per direction.
    Character = 2,
    /// Has sprites for a tile with both animation and direction.
    AnimDir = 3,
    /// Has sprites for a tile with an animation.
    Animated = 4,
}

#[cfg(feature = "serde")]
macro_rules! de_throw {
    ($($tt: tt)+) => {
        ::serde::de::Error::custom(format!($($tt)+))
    };
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
/// A tile's color.
pub enum Color {
    /// Take color from the global palette.
    Paletted { x: u8, y: u8 },
    /// Directly set color to an RGB value.
    RGB { r: u8, g: u8, b: u8 },
}

impl Default for Color {
    fn default() -> Self {
        Color::Paletted { x: 0, y: 3 }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Color::Paletted {x, y} => write!(f, "({x}, {y})"),
            Color::RGB {r, g, b} => write!(f, "#{r:02X}{g:02X}{b:02X}")
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Display)]
/// Something went wrong wile parsing a color.
pub enum ColorError {
    /// RGB color string must be exactly 7 characters long
    WrongLength,
    /// RGB color string must be in base 16
    NotHex,
    #[displaydoc("{0} is not a valid color name")]
    /// This is not a valid color name
    InvalidName(String)
}

impl FromStr for Color {
    type Err = ColorError;

    fn from_str(v: &str) -> Result<Self, Self::Err> {
        // Check for #RRGGBB
        if v.starts_with('#') {
            if v.len() != 7 {
                return Err(ColorError::WrongLength)
            }
            // Deconstruct the literal into bytes
            let rgb = v.strip_prefix('#').unwrap();
            let rgb = u32::from_str_radix(rgb, 16).map_err(
                |_| ColorError::NotHex
            )?;
            let [r, g, b, _] = rgb.to_be_bytes();
            return Ok(Color::RGB { r, g, b });
        }
        // Find colors by name
        Ok( match v {
            // Custom color names
            "maroon"        => Color::Paletted { x: 2, y: 1 },
            "gold"          => Color::Paletted { x: 6, y: 2 },
            "teal"          => Color::Paletted { x: 1, y: 2 },
            // Vanilla color names
            "red"           => Color::Paletted { x: 2, y: 2 },
            "orange"        => Color::Paletted { x: 2, y: 3 },
            "yellow"        => Color::Paletted { x: 2, y: 4 },
            "lime"          => Color::Paletted { x: 5, y: 3 },
            "green"         => Color::Paletted { x: 5, y: 2 },
            "cyan"          => Color::Paletted { x: 1, y: 4 },
            "blue"          => Color::Paletted { x: 3, y: 2 },
            "purple"        => Color::Paletted { x: 3, y: 1 },
            "pink"          => Color::Paletted { x: 4, y: 1 },
            "rosy"          => Color::Paletted { x: 4, y: 2 },
            "grey" | "gray" => Color::Paletted { x: 0, y: 1 }, // aliased
            "black"         => Color::Paletted { x: 0, y: 4 },
            "silver"        => Color::Paletted { x: 0, y: 2 },
            "white"         => Color::Paletted { x: 0, y: 3 },
            "brown"         => Color::Paletted { x: 6, y: 1 },
            // Holdovers from RIC
            "darkpink"      => Color::RGB { r: 0x80, g: 0x00, b: 0x3B },
            // No name matched
            _ => return Err(ColorError::InvalidName(v.to_string()))
        } )
    }
}

#[cfg(feature = "serde")]
impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match self {
            Color::Paletted { x, y } => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element(x)?;
                seq.serialize_element(y)?;
                seq.end()
            },
            Color::RGB { r, g, b } => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element(r)?;
                seq.serialize_element(g)?;
                seq.serialize_element(b)?;
                seq.end()
            }
        }
    }
}

#[cfg(feature = "serde")]
struct ColorVisitor;

#[cfg(feature = "serde")]
impl<'de> de::Visitor<'de> for ColorVisitor {
    type Value = Color;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("a color as an array of 2 or 3 numbers")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
        static WRONG_LEN: &str = "array has a wrong number of arguments for color (expected 2 or 3)";
        // Match the 2 or 3 elements in the array
        // We checked the length earlier, this is fine
        let el_1 = seq.next_element::<u8>()?.ok_or(de_throw!("{WRONG_LEN}"))?;
        let el_2 = seq.next_element::<u8>()?.ok_or(de_throw!("{WRONG_LEN}"))?;
        let el_3 = seq.next_element::<u8>()?;
        if seq.next_element::<u8>()?.is_some() {
            return Err(de_throw!("{WRONG_LEN}"))
        }
        let elements = (el_1, el_2, el_3);
        // Parse a color
        Ok(match elements {
            (r, g, Some(b)) => Color::RGB { r, g, b },
            (x, y, None) => Color::Paletted { x, y }
        })
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_seq(ColorVisitor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// Holds all of the data that a tile has.
pub struct TileData {
    /// The color of the tile
    pub color: Color,
    /// The sprite filename of the tile
    pub sprite: String,
    /// The directory that the tile resides in
    // This shouldn't be specified in a sprites.toml,
    // but should be in the binary database dump.
    #[cfg_attr(feature = "serde", serde(default))]
    pub directory: String,
    /// The tiling mode that the tile uses
    pub tiling: Tiling,
    /// Who created the tile
    pub author: String,
    /// The tile's index into Baba Is You's internal tile grid
    #[cfg_attr(feature = "serde", serde(default))]
    pub tile_index: Option<(u8, u8)>,
    /// The tile's internal object representation in Baba Is You
    #[cfg_attr(feature = "serde", serde(default))]
    pub object_id: Option<String>,
    /// The z layer of this tile (only used in levels)
    #[cfg_attr(feature = "serde", serde(default))]
    pub layer: Option<u8>
}

impl Default for TileData {
    fn default() -> Self {
        Self {
            color: Color::Paletted {x: 0, y: 3},
            sprite: "error".to_string(),
            directory: "vanilla".to_string(),
            tiling: Tiling::None,
            author: "Hempuli".to_string(),
            tile_index: None,
            object_id: None,
            layer: None,
        }
    }
}
