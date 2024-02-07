//! Handles tilemap parsing.
// This is put inside a mod.rs for organization with the pest grammar file.

use std::collections::{BTreeMap, HashMap};
use itertools::Itertools;
use pest::error::{ErrorVariant, Error};
use pest::Parser;
use crate::structures::{Position, Object, Scene, ObjectMap};

mod scene {
    #![allow(missing_docs)]

    use std::fmt::{Display, Formatter};
    use pest_derive::Parser;
    #[derive(Parser)]
    #[grammar = "parser/scene.pest"]
    pub struct SceneParser;

    impl Display for Rule {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", match self {
                Rule::scene => "a scene",
                Rule::flags => "a list of flags",
                Rule::flag => "a flag",
                Rule::tilemap => "a tilemap",
                Rule::row => "a row",
                Rule::stack => "a stack",
                Rule::anim => "an animation cell",
                Rule::cell | Rule::object => "an object",
                Rule::tile => "a tile",
                Rule::variants => "a list of variants",
                Rule::variant => "a variant",
                Rule::vallist => "a list of variant arguments",
                Rule::flag_name => "a name for a flag",
                Rule::flag_arg => "an argument for a flag",
                Rule::var_name => "a name for a variant",
                Rule::var_arg => "a list of arguments for a variant",
                Rule::value | Rule::blacklist | Rule::ws => "<internal token>",
                Rule::EOI => "the end of the input"
            })
        }
    }
}
use scene::Rule;

#[derive(Debug, Clone, PartialEq, Eq)]
/// An unparsed tile.
pub struct RawTile {
    /// The tile's name.
    pub name: String,
    /// The tile's variants.
    pub variants: Vec<RawVariant>
}

impl Object for RawTile {}

#[derive(Debug, Clone, PartialEq, Eq)]
/// An unparsed variant.
pub struct RawVariant {
    /// The variant's name.
    pub name: String,
    /// The variant's arguments.
    pub arguments: Vec<String>
}

/// Formats a pest error for better readability.
fn handle_error(error: Error<Rule>) -> Error<Rule> {
    let ErrorVariant::ParsingError { positives, negatives } = &error.variant else {
        return error;
    };
    // Rule of threes, I'm not factoring this out
    let mut needed = positives
        .iter()
        .map(|rule| format!("{rule}"))
        .collect::<Vec<String>>();
    let unexpected = negatives
        .iter()
        .map(|rule| format!("{rule}"))
        .collect::<Vec<String>>();
    // Construct error messages for both needed and unexpected tokens
    let needed_message = match needed.len() {
        0 => String::new(),
        c => format!("expected {} here\n", match c {
            1 => needed[0].clone(),
            2 => format!("{} or {}", needed.first().unwrap(), needed.get(1).unwrap()),
            _ => {
                let last = needed.pop().unwrap();
                format!("{}, or {}", needed.join(", "), last)
            }
        })
    };
    let unexpected_message = match unexpected.len() {
        0 => String::new(),
        c => format!("did not expect {} here", match c {
            1 => needed[0].clone(),
            2 => format!("{} or {}", needed.first().unwrap(), needed.get(1).unwrap()),
            _ => {
                let last = needed.pop().unwrap();
                format!("{}, or {}", needed.join(", "), last)
            }
        })
    };
    let mut formatted_error = error.clone();
    formatted_error.variant = ErrorVariant::CustomError {
        message: format!("{needed_message}{unexpected_message}")
    };
    formatted_error
}

/// Parses a scene.
///
/// # Errors
/// Errors if the scene fails to parse.
#[allow(clippy::result_large_err)]
pub fn parse(scene: &str) -> Result<Scene<RawTile>, Error<Rule>> {
    // I'll be perfectly honest here.
    // Using pest here is overkill.
    // But, I like using it, so I'm using it.
    let maybe_raw_scene = scene::SceneParser::parse(Rule::scene, scene);
    let Ok(mut raw_scene) = maybe_raw_scene else {
        return Err(handle_error(maybe_raw_scene.unwrap_err()));
    };
    let flags = raw_scene.next().unwrap().into_inner()
        .filter_map(|flag| {
            // Parse an individual flag
            let mut parts = flag.into_inner();
            // .is_empty() for iterators hasn't been stabilized yet
            if parts.len() == 0 { return None; }
            let name = parts.next().unwrap()
                .as_str().to_string();
            let value = parts.next()
                .map(|pair| pair.as_str())
                .map(str::to_string);
            Some((name, value))
        }).collect::<HashMap<_, _>>();
    // Iterator over iterators over (Position, Pair<Rule>)
    let tilemap_iter = raw_scene.next().unwrap()
        .into_inner().enumerate().flat_map(|(y, row)|
        row.into_inner().enumerate().flat_map(move |(x, stack)|
            stack.into_inner().enumerate().map(move |(z, animation)|
                animation.into_inner().enumerate().map(move |(t, cell)|
                    (Position {x: x as i64, y: y as i64, z: z as i64, t: t as i64}, cell)
                )
            )
        )
    );
    let Position {x: width, y: height, t: length, ..} = tilemap_iter
        .clone() // Cloning iterators should be free?
        .flatten()
        .fold(Position::default(), |last, (this, _)|
            Position {
                x: last.x.max(this.x),
                y: last.y.max(this.y),
                z: last.z.max(this.z),
                t: last.t.max(this.t)
            }
        );
    let tiles = tilemap_iter.flat_map(|iter| {
        let mut nonstop_iter = iter.map(|v| Some(v)).chain(
            std::iter::repeat(None)
        );
        // We currently have an iterator over every tile in this animation cell
        nonstop_iter.scan(None, |last_tile: &mut Option<(Position, RawTile)>, maybe_tile| {
            let mut pairs = current_obj.into_inner();
            let object = pairs.next().unwrap();
            let variants = pairs.next().unwrap();
            if object.as_rule() != Rule::tile {
                todo!("Object type not implemented: {}", object.as_rule())
            }
            let name = object.as_str();

            let mut new_tile = false;

            // Get the current tile
            let name = match name {
                // Implicitly empty, fill with last tile
                "" if last_tile.is_some() => last_tile.as_ref()
                    .unwrap()
                    .name.to_string(),
                // Explicitly empty, clear last and return Some(None)
                "." | "" => {
                    *last_tile = None;
                    return Some(None);
                },
                name => {
                    // This is explicitly something new
                    new_tile = true;
                    name.to_string()
                }
            };

            let mut variants = variants.into_inner().map(|variant| {
                let mut variant = variant.into_inner();
                let name = variant.next().unwrap().as_str().to_string();
                let arguments = variant
                    .map(|pair| pair.as_str().to_string())
                    .collect::<Vec<_>>();
                RawVariant { name, arguments }
            }).collect::<Vec<_>>();
            if !new_tile && variants.is_empty() && last_tile.is_some() {
                // Fill the tile's variants with the last tile's variants
                variants = last_tile.as_ref().unwrap().variants.clone();
            }

            let tile = RawTile {name, variants};
            *last_tile = Some((pos, tile.clone()));
            Some(Some((pos, tile)))
        })
    })
        .flatten() // Remove the Nones
        .collect::<BTreeMap<_, _>>();

    Ok(Scene {
        map: ObjectMap {
            width: width as u64,
            height: height as u64,
            length: length as u64,
            objects: tiles,
        },
        flags,
    })
}
