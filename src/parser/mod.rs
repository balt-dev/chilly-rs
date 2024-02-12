//! Handles tilemap parsing.
// This is put inside a bot for organization with the pest grammar file.

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use itertools::Itertools;
use num_traits::Num;
use pest::{
    error::{Error, ErrorVariant},
    iterators::Pair,
    Parser
};
use crate::structures::{
    Position, Object,
    Scene, ObjectMap
};

mod scene {
    #![allow(missing_docs)]

    use std::fmt::{Display, Formatter};
    use pest_derive::Parser;
    #[derive(Parser)]
    #[grammar = "parser/scene.pest"]
    pub struct Parser;

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
                Rule::text | Rule::glyph | Rule::tag => "a tile prefix",
                Rule::tile_name => "a tile name",
                Rule::EOI => "the end of the input"
            })
        }
    }
}
use scene::Rule;
use crate::variants::{Variant, VariantError, VariantName};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
/// A tag for a tile.
#[non_exhaustive]
pub enum TileTag {
    /// Prepends `text_` to a tile's name, or removes it if in text mode alredy.
    Text,
    /// Prepends `glyph_` to a tile's name.
    Glyph
}

#[derive(Debug, Clone, PartialEq)]
/// An unparsed tile.
pub struct RawTile<'scene> {
    /// The tile's name.
    pub name: &'scene str,
    /// The tag the tile may have.
    pub tag: Option<TileTag>,
    /// The tile's variants.
    pub variants: Vec<Variant>
}

impl<'s> Object for RawTile<'s> {}

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

// Helper function to unescape a string
fn unescape(string: &str) -> Cow<str> {
    let fixed = string
        .replace(r"\\", "\x00") //Intermediary
        .replace(r"\n", "\n")
        .replace(r"\r", "\r")
        .replace(r"\t", "\t")
        .replace('\\', "")
        .replace('\x00', r"\"); // Fix intermediary
    if fixed == string {
        Cow::Borrowed(string)
    } else {
        Cow::Owned(fixed)
    }
}

/// Parses a scene.
///
/// # Errors
/// Errors if the scene fails to parse.
#[allow(clippy::result_large_err, clippy::missing_panics_doc)]
pub fn parse(scene: &str) -> Result<Scene<RawTile, usize>, Error<Rule>> {
    // I'll be perfectly honest here.
    // Using pest here is overkill.
    // But, I like using it, so I'm using it.
    let maybe_raw_scene = scene::Parser::parse(Rule::scene, scene);
    let Ok(mut raw_scene) = maybe_raw_scene else {
        return Err(handle_error(maybe_raw_scene.unwrap_err()));
    };
    let flags = raw_scene.next().unwrap().into_inner()
        .filter_map(|flag| {
            // Parse an individual flag
            let mut parts = flag.into_inner();
            // .is_empty() for iterators hasn't been stabilized yet
            if parts.len() == 0 { return None; }
            // TODO: These could be Cow<str>
            let name = unescape(parts.next().unwrap().as_str()).into_owned();
            let value = parts.next()
                .map(|pair|
                     pair.as_str().to_string()
                );
            Some((name, value))
        }).collect::<HashMap<_, _>>();
    // Iterator over iterators over (Position, Pair<Rule>)
    let tilemap_iter = raw_scene.next().unwrap()
        .into_inner().enumerate().flat_map(|(y, row)|
        row.into_inner().enumerate().flat_map(move |(x, stack)|
            stack.into_inner().enumerate().map(move |(z, animation)|
                animation.into_inner().enumerate().map(move |(t, cell)|
                    (Position {x, y, z, t}, cell)
                )
            )
        )
    );
    let Position {x: width, y: height, t: length, ..} = tilemap_iter
        .clone() // Cloning iterators should be free?
        .flatten()
        .fold(Position::<usize>::default(), |last, (this, _)|
            Position {
                x: last.x.max(this.x + 1),
                y: last.y.max(this.y + 1),
                z: last.z.max(this.z + 1),
                t: last.t.max(this.t + 1)
            }
        );
    let tiles = tilemap_iter.flat_map(|iter| {
        let nonstop_iter = iter.map(Some).pad_using(length, |_| None);
        // We currently have an iterator over every tile in this animation cell
        nonstop_iter.scan(None, |last_tile: &mut Option<(Position<usize>, RawTile)>, maybe_tile| {
            if maybe_tile.is_none() {
                // Reached the end but there's still more frames
                // Fill with the last, if it exists
                let mut last = last_tile.clone();
                if let Some((ref mut pos, _)) = last {
                    pos.t += 1; // Increment the frame counter so it's not on the same frame
                }
                *last_tile = last.clone();
                return Some(Ok(last))
            }
            let (pos, current_obj) = maybe_tile.unwrap();
            let mut pairs = current_obj.into_inner();
            let object = pairs.next().unwrap();
            let obj_span = object.as_span();
            let variants = pairs.next().unwrap();
            // Check what this object actually is
            if object.as_rule() != Rule::tile {
                todo!("Object type not implemented: {}", object.as_rule())
            }
            let mut parts = object.into_inner();
            let mut tag = parts.next().unwrap().into_inner();
            let tag = tag.next().map(|pair| match pair.as_rule() {
                Rule::text => TileTag::Text,
                Rule::glyph => TileTag::Glyph,
                _ => unreachable!()
            });
            let name = parts.next().unwrap().as_str();

            // Parse the tile
            let parsed = parse_tile(last_tile, tag, name, variants);
            let Ok(parsed) = parsed else {
                let err = parsed.unwrap_err();
                return Some(Err(Error::new_from_span(
                    ErrorVariant::CustomError {
                        message: format!("{err}")
                    },
                    obj_span
                )));
            };
            let Some(tile) = parsed else {
                return Some(Ok(None));
            };
            *last_tile = Some((pos, tile.clone()));
            Some(Ok(Some((pos, tile))))
        })
    })
        .filter(|res| res.as_ref().is_ok_and(Option::is_some))
        .map(|res| res.map(|opt| opt.expect("we filtered out the Nones earlier")))// Remove the Nones
        .collect::<Result<BTreeMap<_, _>, Error<Rule>>>()?;

    Ok(Scene {
        map: ObjectMap {
            width,
            height,
            length,
            objects: tiles,
        },
        flags,
    })
}

fn parse_tile<'scene, N: Num>(
    last_tile: &mut Option<(Position<N>, RawTile<'scene>)>,
    tag: Option<TileTag>,
    name: &'scene str,
    variants: Pair<'scene, Rule>
) -> Result<Option<RawTile<'scene>>, VariantError> {
    let mut new_tile = false;

    let name = match name {
        // Implicitly empty, fill with last tile
        "" if last_tile.is_some() =>
            last_tile.as_ref().unwrap().1.name,
        // Explicitly empty, clear last and return Some(None)
        "." | "" => {
            *last_tile = None;
            return Ok(None);
        },
        name => {
            // This is explicitly something new
            new_tile = true;
            name
        }
    };

    // Parse variants
    let mut variants = variants.into_inner().map(|variant| {
        let mut variant = variant.into_inner();

        let name: &'scene str = variant.next().unwrap().as_str();
        let arguments = variant
            .map(|pair| pair.as_str());

        let identifier = VariantName::from_alias(name)?;
        Variant::parse(
            identifier, arguments
        )
    }).collect::<Result<Vec<_>, VariantError>>()?;
    if !new_tile && variants.is_empty() && last_tile.is_some() {
        // Fill the tile's variants with the last tile's variants
        variants = last_tile.as_ref().unwrap().1.variants.clone();
    }

    Ok(Some(RawTile::<'scene> {name, tag, variants}))
}
