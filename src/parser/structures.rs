use std::collections::HashMap;
use pest::Span;

use crate::{
    structures::{Object, ObjectMap},
    variants::Variant
};


/// A raw scene, before any parsing efforts.
#[derive(Debug, Clone)]
pub struct RawScene<'scene> {
    /// A tilemap of the objects in the scene.
    pub map: ObjectMap<RawTile<'scene>, usize>,
    /// The attached flags of the scene.
    pub flags: HashMap<String, Option<String>>
}


#[derive(Debug, Clone, PartialEq)]
/// An unparsed tile.
pub struct RawTile<'scene> {
    /// The tile's name.
    pub name: &'scene str,
    /// The tag the tile may have.
    pub tag: Option<TileTag>,
    /// The tile's variants.
    pub variants: Vec<Variant>,
    /// The span of the tile's name. Used for error reporting.
    pub(crate) span: Span<'scene>
}

impl<'s> Object for RawTile<'s> {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// A tag for a tile.
#[non_exhaustive]
pub enum TileTag {
    /// Prepends `text_` to a tile's name, or removes it if in text mode alredy.
    Text,
    /// Prepends `glyph_` to a tile's name.
    Glyph
}
