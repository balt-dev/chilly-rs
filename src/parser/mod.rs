//! Handles tilemap parsing.
// This is put inside a bot for organization with the pest grammar file.

mod structures;
pub use structures::{
    TileTag,
    RawScene,
    RawTile
};

use std::{
    borrow::Cow,
    collections::{HashMap, HashSet}
};
use itertools::Itertools;
use num_traits::Num;
use pest::{
    error::{Error, ErrorVariant},
    iterators::Pair,
    Parser
};
use crate::{arguments::{Flag, FlagName}, structures::{
    ObjectMap, Position
}};

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
pub(crate) use scene::Rule;
use crate::arguments::{Variant, ArgumentError, VariantName};

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

/// Parses a raw scene.
///
/// # Errors
/// Errors if the scene fails to parse.
#[allow(clippy::result_large_err, clippy::missing_panics_doc)]
pub fn parse(scene: &str) -> Result<RawScene, Error<Rule>> {
    // I'll be perfectly honest here.
    // Using pest here is overkill.
    // But, I like using it, so I'm using it.
    let maybe_raw_scene = scene::Parser::parse(Rule::scene, scene);
    let Ok(mut raw_scene) = maybe_raw_scene else {
        return Err(handle_error(maybe_raw_scene.unwrap_err()));
    };
    let flags: HashMap<FlagName, Flag> = raw_scene.next().unwrap().into_inner()
        .filter_map(|flag| {
            // Parse an individual flag
            let mut parts = flag.into_inner();
            // .is_empty() for iterators hasn't been stabilized yet
            if parts.len() == 0 { return None; }
            // TODO: These could be Cow<str>
            // Parse the name and arguments of the flag
            let name_pair = parts.next().unwrap();
            let name = name_pair.as_str();
            let args: Vec<_> = parts.collect();
            let arg_strings = args.iter().map(Pair::as_str);
            let mut arg_spans = args.iter().map(Pair::as_span);
            // Parse the name
            let identifier = FlagName::from_alias(name).ok_or_else(||
                Error::new_from_span(
                    ErrorVariant::CustomError { message: format!("flag \"{name}\" does not exist") },
                    name_pair.as_span()
                )
            );
            let Ok(identifier) = identifier else {return Some(Err(identifier.unwrap_err()))};
            let flag = Flag::parse(identifier, arg_strings).map_err(|err| {
                let ArgumentError::InvalidArgument("flag", idx, err) = err 
                    else {unreachable!("invalid flag should be the only error passed back here")};
                let span = arg_spans.nth(idx).unwrap_or(name_pair.as_span());
                Error::new_from_span(
                    ErrorVariant::CustomError { message: format!("failed to parse flag: {err}") },
                    span
                )
            });
            let Ok(flag) = flag else {return Some(Err(flag.unwrap_err()))};
            Some(Ok((identifier, flag)))
        }).collect::<Result<_, _>>()?;
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
            let name = parts.next().unwrap();

            // Parse the tile
            let parsed = parse_tile(last_tile, tag, &name, variants);
            let Ok(parsed) = parsed else {
                let err = parsed.unwrap_err();
                return Some(Err(err));
            };
            let Some(tile) = parsed else {
                return Some(Ok(None));
            };
            *last_tile = Some((pos, tile.clone()));
            Some(Ok(Some((pos, tile))))
        })
    })
        .filter(|res| {
            let res = res.as_ref();
            res.is_err() || res.is_ok_and(Option::is_some)
        })
        .map(|res| res.map(|opt| opt.expect("we filtered out the Nones earlier")))// Remove the Nones
        .collect::<Result<_, Error<Rule>>>()?;

    Ok(RawScene {
        map: ObjectMap {
            width,
            height,
            length,
            objects: tiles,
        },
        flags,
    })
}

#[allow(clippy::result_large_err)]
fn parse_tile<'scene, N: Num>(
    last_tile: &mut Option<(Position<N>, RawTile<'scene>)>,
    tag: Option<TileTag>,
    name: &Pair<'scene, Rule>,
    variants: Pair<'scene, Rule>
) -> Result<Option<RawTile<'scene>>, Error<Rule>> {
    let mut new_tile = false;

    let name_string = match name.as_str() {
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

        let name_pair = variant.next().unwrap();
        let name: &'scene str = name_pair.as_str();
        let args: Vec<_> = variant.collect();
        let arg_strings = args.iter().map(Pair::as_str);
        let mut arg_spans = args.iter().map(Pair::as_span);

        if let Some(var) = Variant::collapse_alias(name) {
            Ok(var)
        } else {
            let identifier = VariantName::from_alias(name).ok_or_else(||
                Error::new_from_span(
                    ErrorVariant::CustomError { message: format!("variant \"{name}\" does not exist") },
                    name_pair.as_span()
                )
            )?;
            Variant::parse(
                identifier, arg_strings
            ).map_err(|err| {
                let ArgumentError::InvalidArgument("variant", idx, err) = err 
                    else {unreachable!("invalid argument should be the only error passed back here")};
                let span = arg_spans.nth(idx).unwrap_or(name_pair.as_span());
                Error::new_from_span(
                    ErrorVariant::CustomError { message: format!("failed to parse variant: {err}") },
                    span
                )
            })
        }


    }).collect::<Result<Vec<_>, Error<Rule>>>()?;
    if !new_tile && variants.is_empty() && last_tile.is_some() {
        // Fill the tile's variants with the last tile's variants
        variants = last_tile.as_ref().unwrap().1.variants.clone();
    }

    Ok(Some(RawTile::<'scene> {name: name_string, tag, variants, span: name.as_span()}))
}
