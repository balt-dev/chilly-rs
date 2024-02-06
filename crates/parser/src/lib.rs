#![warn(missing_docs, clippy::pedantic, clippy::perf)]
#![doc = include_str!(r"../README.md")]

use std::collections::HashMap;
use pest::error::ErrorVariant;
use pest::iterators::Pair;
use pest::Parser;
use structures::{Position, Object, Scene, ObjectMap};

mod scene {
    #![allow(missing_docs)]

    use std::fmt::{Display, Formatter};
    use pest_derive::Parser;
    #[derive(Parser)]
    #[grammar = "scene.pest"]
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
use scene::*;

type ParseResult<T> = Result<T, pest::error::Error<Rule>>;

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
fn handle_error(error: pest::error::Error<Rule>) -> pest::error::Error<Rule> {
    let ErrorVariant::ParsingError { positives, negatives } = error else {
        return error;
    };
    // Rule of threes, I'm not factoring this out
    let mut needed = positives
        .into_iter()
        .map(|rule: Rule| format!("{rule}"))
        .collect::<Vec<_>>();
    let mut unexpected = negatives
        .into_iter()
        .map(|rule: Rule| format!("{rule}"))
        .collect::<Vec<_>>();
    // Construct error messages for both needed and unexpected tokens
    let needed_message = match needed.len() {
        0 => String::new(),
        c => format!("expected {} here\n", match c {
            1 => needed.get(0).unwrap(),
            2 => format!("{} or {}", needed.get(0).unwrap(), needed.get(1).unwrap()),
            _ => {
                let last = needed.pop().unwrap();
                format!("{}, or {}", needed.join(", "), last)
            }
        })
    };
    let unexpected_message = match unexpected.len() {
        0 => String::new(),
        c => format!("did not expect {} here", match c {
            1 => needed.get(0).unwrap(),
            2 => format!("{} or {}", needed.get(0).unwrap(), needed.get(1).unwrap()),
            _ => {
                let last = needed.pop().unwrap();
                format!("{}, or {}", needed.join(", "), last)
            }
        })
    };
    return pest::error::Error {
        variant: ErrorVariant::CustomError {
            message: format!("{needed_message}{unexpected_message}")
        },
        ..error
    }
}

/// Parses a scene.
/// 
/// # Errors
/// Errors if the scene fails to parse.
pub fn parse(scene: &str) -> ParseResult<Scene<RawTile>> {
    // I'll be perfectly honest here.
    // Using pest here is overkill.
    // But, I like using it, so I'm using it.
    let mut maybe_raw_scene = scene::SceneParser::parse(Rule::scene, scene);
    let Ok(mut raw_scene) = maybe_raw_scene else {
        return Err(handle_error(maybe_raw_scene.unwrap_err()));
    };
    let flags = raw_scene.next().unwrap().into_inner()
        .map(|flag| {
            // Parse an individual flag
            let mut parts = flag.into_inner();
            let name = parts.next().unwrap()
                .as_str().to_string();
            let value = parts.next()
                .map(|pair| pair.as_str())
                .map(str::to_string);
            (name, value)
        }).collect::<HashMap<_, _>>();
    // Iterator over (Position, Pair<Rule>)
    let mut tilemap_iter = raw_scene.next().unwrap()
        .into_inner().into_iter().enumerate().flat_map(|(y, row)|
            row.into_inner().into_iter().enumerate().flat_map(|(x, stack)|
                stack.into_inner().into_iter().enumerate().flat_map(|(z, animation)|
                    animation.into_inner().into_iter().enumerate().map(|(t, cell)|
                        (Position {x: x as i64, y: y as i64, z: z as i64, t: t as i64}, cell)
                    )
                )
            )
        );
    let Position{x: width, y: height, t: length, .. } = tilemap_iter.by_ref().fold(
            Position::default(),
            |pos1, (pos2, _)|
                Position {
                    x: pos1.x.max(pos2.x),
                    y: pos1.y.max(pos2.y),
                    z: pos1.z.max(pos2.z),
                    t: pos1.t.max(pos2.t)
                }
    );
    let width = width as u64 + 1;
    let height = height as u64 + 1;
    let length = length as u64 + 1;
    let tiles = tilemap_iter.filter_map(|(pos, cell)| {
        let mut cell_tokens = cell.into_inner();
        if cell_tokens.len() == 0 {
            return None;
        }
        let object = cell_tokens.next().unwrap();
        let variants = cell_tokens.next().unwrap();

        todo!("Object parsing")
    });
    todo!("Parse")
}
