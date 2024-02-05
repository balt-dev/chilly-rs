#![warn(missing_docs, clippy::pedantic, clippy::perf)]
#![doc = include_str!(r"../README.md")]

use pest::Parser;
use structures::{Position, Object, Scene, ObjectMap};

mod scene {
    #![allow(missing_docs)]

    use pest_derive::Parser;
    #[derive(Parser)]
    #[grammar = "scene.pest"]
    pub struct SceneParser;
}

type ParseResult<T> = std::result::Result<T, pest::error::Error<scene::Rule>>;

#[derive(Debug, Clone, PartialEq, Eq)]
/// An unparsed tile.
pub struct RawTile {
    /// The tile's name.
    pub name: String,
    /// The tile's variants.
    pub variants: Vec<RawVariant>
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// An unparsed variant.
pub struct RawVariant {
    /// The variant's name.
    pub name: String,
    /// The variant's arguments.
    pub arguments: Vec<String>
}

/// Parses a scene.
/// 
/// # Errors
/// Errors if the scene fails to parse.
pub fn parse(scene: &str) -> ParseResult<Scene<RawTile>> {
    let mut maybe_raw_scene = scene::SceneParser::parse(scene::Rule::scene, scene);
    let Ok(raw_scene) = maybe_raw_scene else {
        todo!("Handle")
    };
    todo!("Parse")
}