#![cfg(feature = "rendering")]
//! Handles rendering of scenes into sprites.

use crate::{database::structures::Color, solidify::{SkeletalScene, TileSkeleton, TileSkeletonType}, structures::Position};
use image::{io::Reader as ImageReader, Rgba, RgbaImage};
use pest::Span;
use try_insert_ext::EntryInsertExt;
use std::{
    borrow::Cow,
    collections::HashMap,
    io,
    path::{Path, PathBuf},
};

mod structures;
use crate::arguments::{Flag, FlagName};
pub use structures::{RenderedScene, RenderingError};

use self::structures::{RawSprite, Sprite};

/// Opens an image, potentially from a cache.
/// 
/// # Note
/// You may have lifetime issues with this funciton. 
/// 
/// The pointer for a borrow would dangle or break aliasing rules if the cache is changed after returning,
/// so before any subsequent calls, you must clone the values you get from this function,
/// or drop the return value entirely.
fn open_cached(
    path: impl AsRef<Path>,
    cache: Option<&mut HashMap<PathBuf, RgbaImage>>,
) -> Result<Cow<RgbaImage>, io::Error> {
    let path = path.as_ref();
    // Create a closure so we don't have to duplicate code
    let imgen = || {
        Ok::<_, io::Error>( 
            ImageReader::open(path)?
                .decode()
                .map_err(io::Error::other)?
                .into_rgba8() 
        )
    };
    // Infallibly match to hint to the borrow checker that we can't go past here
    let None = cache else {
        let cache = cache.unwrap();
        let cache_entry = cache.entry(path.to_path_buf()).or_try_insert_with(imgen)?;
        return Ok(Cow::Borrowed(cache_entry));
    };
    let image = imgen()?;
    Ok(Cow::Owned(image))
}

fn resolve_palette<'s>(
    asset_path: PathBuf,
    palette_path: PathBuf,
    cache: Option<&mut HashMap<PathBuf, RgbaImage>>,
) -> Result<Cow<RgbaImage>, RenderingError<'s>> {
    // Resolve the palette
    let mut glob_pattern = asset_path;
    glob_pattern.push("*");
    glob_pattern.push(palette_path.to_string_lossy().as_ref());
    let glob_pattern = glob_pattern
        .with_extension("png")
        .to_string_lossy()
        .to_string();
    let found_palette_path = glob::glob(&glob_pattern)
        .expect("glob pattern was invalid")
        .find_map(Result::ok);
    let Some(path) = found_palette_path else {
        return Err(RenderingError::NoPalette(palette_path));
    };
    let reader =
        ImageReader::open(&path).map_err(|err| RenderingError::FailedOpen(path.clone(), err))?;
    let image = reader
        .decode()
        .map_err(|err| RenderingError::FailedDecode(path, err))?
        .into_rgba8();
    if let Some(cache) = cache {
        Ok(Cow::Borrowed(cache.entry(palette_path).or_insert(image)))
    } else {
        Ok(Cow::Owned(image))
    }
}





/// The main entrypoint in the renderer.
///
/// If a cache can be passed in, then paths
/// from the cache will be used instead of
/// opening the file at the path.
///
/// # Notes
/// This is __not guaranteed__ to return the sprites of each frame in sorted order.
/// However, the Z order of every sprite is guaranteed to be unique for its frame.
/// Using [`sort_unstable_by`](::core::slice::sort_unstable_by) is recommended.
///
/// # Errors
/// Errors if the scene fails to render. See [`RenderingError`] for details.
pub fn render<'scene, 'cache>(
    mut scene: SkeletalScene<'_, 'scene>,
    asset_path: impl AsRef<Path>,
    cache: Option<&'cache mut HashMap<PathBuf, RgbaImage>>,
) -> Result<RenderedScene<'cache>, RenderingError<'scene>> {
    let asset_path = asset_path.as_ref();

    // Parse boolean flags
    let loops = scene.flags.remove(&FlagName::NoLoop).is_none();

    // Get palette from assets
    let palette_path: PathBuf = if let Some(flag) = scene.flags.remove(&FlagName::Palette) {
        let Flag::Palette(path) = flag else {
            unreachable!()
        };
        path.into()
    } else {
        "default".into()
    };

    let frame_indices = if let Some(flag) = scene.flags.remove(&FlagName::WobbleFrames) {
        let Flag::WobbleFrames(wobbles) = flag else {
            unreachable!()
        };
        if wobbles.is_empty() {
            return Err(RenderingError::InvalidFlag(FlagName::WobbleFrames, "must have at least one wobble frame".into()))
        }
        if wobbles.iter().any(|frame| !(1..=3).contains(frame)) {
            return Err(RenderingError::InvalidFlag(FlagName::WobbleFrames, "all wobble frames must be between 1 and 3".into()))
        }
        wobbles
    } else {
        vec![1, 2, 3]
    };

    let anim_frames_per_wobble = if let Some(flag) = scene.flags.remove(&FlagName::DecoupleWobble) {
        let Flag::DecoupleWobble(anim, _) = flag else {
            unreachable!()
        };
        if anim == 0 {
            return Err(RenderingError::InvalidFlag(FlagName::DecoupleWobble, "cannot have 0 animation frames per wobble frame (would lead to div. by 0)".into()))
        }
        anim as usize
    } else {
        frame_indices.len()
    };

    let palette = cache.as_ref().and_then(|cache| cache.get(&palette_path));
    // We use a match here to allow error propogation
    let palette = match palette {
        Some(v) => Cow::Borrowed(v),
        None => resolve_palette(asset_path.to_path_buf(), palette_path, cache)?,
    };
    // Get background color
    let background_color = scene
        .flags
        .remove(&FlagName::BackgroundColor)
        .and_then(|flag| {
            let Flag::BackgroundColor(color) = flag else {
                unreachable!()
            };
            color
                .unwrap_or(
                    Color::Paletted { x: 0, y: 0 }, // Default background color
                )
                .into_rgba(palette.as_ref())
        })
        .unwrap_or(Rgba([0; 4]));

    // See the documentation for open_cached for why we do this
    let palette = palette.into_owned();
    
    // Convert all tile skeletons to sprites
    let sprites = scene.map.objects.into_iter()
        .map(|(pos, skel)| handle_sprite(
            asset_path, cache, pos, skel, anim_frames_per_wobble, frame_indices
        ))
        .collect::<Result<Vec<Sprite>, _>>()?;
    todo!()
}

/// Converts a single [`TileSkeleton`] into a [`Sprite`].
/// 
/// # Errors
/// Errors if conversion failed. 
fn handle_sprite<'path, 'cache, 'scene>(
    asset_path: &'path Path,
    cache: Option<&'cache mut HashMap<PathBuf, RgbaImage>>,
    pos: Position<usize>,
    skel: TileSkeleton,
    anim_frames_per_wobble: usize,
    frame_indices: Vec<u8>
) -> Result<Sprite<'cache>, RenderingError<'scene>> {
    let time_index = pos.t;
    let frame_index = (time_index / anim_frames_per_wobble) % frame_indices.len();
    // Due to doing % len, this is guaranteed to exist,
    // unless we're given a length of 0, which is handled in the flag parsing
    let wobble_frame = frame_indices[frame_index];

    let sprite = match skel.data {
        TileSkeletonType::Existing(existing) => {
            // Construct the sprite path
            let sprite_name = &existing.sprite;
            let mut sprite_path = asset_path.to_path_buf();
            sprite_path.push(&existing.directory);
            sprite_path.push("sprites");
            // Create a fallback path to check if the current path doesn't exist
            let mut fallback_path = sprite_path.clone();
            sprite_path.push(format!("{}_{}_{}.png", existing.sprite, skel.animation_frame.0, wobble_frame));
            fallback_path.push(format!("{}_{}_{}.png", existing.sprite, skel.animation_frame.1, wobble_frame));
            match open_cached(&sprite_path, cache) {
                // Found the default sprite - return it
                Ok(v) => RawSprite {
                    image: v.into_owned(),
                    color: existing.color
                },
                // Couldn't find default sprite - try the fallback
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    let fallback = open_cached(fallback_path, cache)
                        .map_err(|e| RenderingError::SpriteFailedOpen(skel.span, e))?.into_owned();
                    // Add the fallback to the cache in the original's stead
                    if let Some(cache) = cache {
                        cache.insert(sprite_path, fallback);
                    }
                    RawSprite {
                        image: fallback,
                        color: existing.color
                    }
                },
                // Found it, but something else happened - reraise
                Err(e) => return Err(RenderingError::SpriteFailedOpen(skel.span, e))
            }
        },
        TileSkeletonType::Generative(gen) => generate_sprite(asset_path, cache, gen, skel.span)?
    };

    todo!()
}

/// Generates a sprite from a string.
fn generate_sprite<'path, 'cache, 'scene>(
    path: &'path Path,
    cache: Option<&'cache mut HashMap<PathBuf, RgbaImage>>,
    genstring: Cow<'scene, str>,
    span: Span<'scene>
) -> Result<RawSprite, RenderingError<'scene>> {
    Err(RenderingError::SpriteNoTile(span, genstring))
}