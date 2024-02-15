#![cfg(feature = "rendering")]
//! Handles rendering of scenes into sprites.

use crate::{database::structures::Color, solidify::{SkeletalScene, TileSkeleton, TileSkeletonType}, structures::Position};
use image::{DynamicImage, ImageBuffer, io::Reader as ImageReader, Luma, Rgba, RgbaImage};
use pest::Span;
use try_insert_ext::EntryInsertExt;
use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
    hash::BuildHasher
};
use imageproc::definitions::Image;
use imageproc::filter::Kernel;
use itertools::Itertools;

mod structures;
use crate::arguments::{Flag, FlagName, MetaKernel, Variant, VariantName};
pub use structures::{RenderedScene, RenderingError};

use self::structures::{RawSprite, Sprite};

/// Opens an image, potentially from a cache.
/// 
/// # Note
/// The pointer for a borrow would dangle or break aliasing rules if the cache is changed after returning,
/// so this sadly cannot return a [`Cow`].
///
/// (Or at least it probably could, but I'm not dealing with 50 different lifetimes at once.)
fn open_cached<S: BuildHasher>(
    path: impl AsRef<Path>,
    cache: &mut Cache<S>,
) -> Result<RgbaImage, io::Error> {
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
        let cache = cache.as_mut().unwrap();
        let cache_entry = cache.entry(path.to_path_buf()).or_try_insert_with(imgen)?;
        return Ok(cache_entry.clone());
    };
    let image = imgen()?;
    Ok(image)
}

fn resolve_palette<'s, 'br, 'c: 'br, S: BuildHasher>(
    asset_path: PathBuf,
    palette_path: PathBuf,
    cache: &'br mut Cache<'s, S>,
) -> Result<RgbaImage, RenderingError<'s>> {
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
        Ok(cache.entry(palette_path).or_insert(image).clone())
    } else {
        Ok(image)
    }
}



type Cache<'c, S> = Option<&'c mut HashMap<PathBuf, RgbaImage, S>>;

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
pub fn render<'scene, 'cache, S: BuildHasher>(
    mut scene: SkeletalScene<'_, 'scene>,
    asset_path: impl AsRef<Path>,
    mut cache: Cache<'scene, S>,
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

    let frames_per_wobble = if let Some(flag) = scene.flags.remove(&FlagName::DecoupleWobble) {
        let Flag::DecoupleWobble(_, wobble) = flag else {
            unreachable!()
        };
        if wobble == 0 {
            return Err(RenderingError::InvalidFlag(FlagName::DecoupleWobble, "cannot have 0 frames per wobble frame (would lead to div. by 0)".into()))
        }
        wobble as usize
    } else {
        frame_indices.len()
    };

    let palette = cache.as_ref().and_then(|cache| cache.get(&palette_path));
    // We use a match here to allow error propogation
    let palette = match palette {
        Some(v) => v.clone(),
        None => resolve_palette(asset_path.to_path_buf(), palette_path, &mut cache)?,
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
                .into_rgba(&palette)
        })
        .unwrap_or(Rgba([0; 4]));
    
    // Convert all tile skeletons to sprites
    let sprites = scene.map.objects.into_iter()
        .map(|(pos, skel)| handle_sprite(
            asset_path, &mut cache, pos, skel, frames_per_wobble, &frame_indices
        ))
        .collect::<Result<Vec<Sprite>, _>>()?;
    todo!()
}

/// Converts a single [`TileSkeleton`] into a [`Sprite`].
/// 
/// # Errors
/// Errors if conversion failed. 
fn handle_sprite<'cache, 'scene, S: BuildHasher>(
    asset_path: &Path,
    cache: &mut Cache<'scene, S>,
    pos: Position<usize>,
    mut skel: TileSkeleton<'_, 'scene>,
    frames_per_wobble: usize,
    frame_indices: &[u8]
) -> Result<Sprite<'cache>, RenderingError<'scene>> {
    let time_index = pos.t;
    let frame_index = (time_index / frames_per_wobble) % frame_indices.len();
    // Due to doing % len, this is guaranteed to exist,
    // unless we're given a length of 0, which is handled in the flag parsing
    let wobble_frame = frame_indices[frame_index];

    let sprite = match skel.data {
        TileSkeletonType::Existing(existing) => {
            // Construct the sprite path
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
                    image: v,
                    color: existing.color
                },
                // Couldn't find default sprite - try the fallback
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    let mut fallback = open_cached(fallback_path, cache)
                        .map_err(|e| RenderingError::SpriteFailedOpen(skel.span, e))?;
                    // Add the fallback to the cache in the original's stead
                    if let Some(cache) = cache {
                        fallback = cache.entry(sprite_path).or_insert(fallback).clone();
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
        TileSkeletonType::Generative(ref gen) => generate_sprite(asset_path, cache, gen.to_string(), skel.span)?
    };

    let sprite = handle_sprite_variants(asset_path, cache, &mut skel, sprite)?;

    todo!()
}

/// Generates a sprite from a string.
fn generate_sprite<'scene, S: BuildHasher>(
    path: &Path,
    cache: &mut Cache<S>,
    genstring: String,
    span: Span<'scene>
) -> Result<RawSprite, RenderingError<'scene>> {
    Err(RenderingError::SpriteNoTile(span, genstring))
}

macro_rules! variant_assert {
    ($name: ident @ $span: expr; $check: expr; $($message: tt)+) => {
        if !{$check} {
            return Err(
                RenderingError::SpriteInvalidVariant(
                    $span, VariantName::$name, format!($($message)+)
                )
            )
        }
    };
}

/// Handles any variants that a sprite has, changing the sprite if need be.
///
/// # Errors
/// Errors if a sprite variant fails to apply. See [`RenderingError`] for more details.
fn handle_sprite_variants<'scene, S: BuildHasher>(
    path: &Path,
    cache: &mut Cache<S>,
    skel: &mut TileSkeleton<'_, 'scene>,
    mut raw_sprite: RawSprite
) -> Result<RawSprite, RenderingError<'scene>> {
    let variants = &mut skel.variants;
    let mut new_variants = Vec::new();
    for variant in variants.drain(..) {
        match variant {
            Variant::Meta(level, kernel, size) => {
                // Unwrap defaults
                let level = level.unwrap_or(1);
                let kernel = kernel.unwrap_or(MetaKernel::Full);
                let size = size.unwrap_or(1);
                variant_assert!(Meta @ skel.span; size != 0; "meta size can't be zero");
                // Extract alpha channel and turn it into an image we can convolve
                let base = raw_sprite.image.pixels().map(
                    |pix| {
                        let alpha = pix.0[3];
                        if level < 0 { !alpha } else { alpha }
                    }
                ).collect::<Vec<_>>();
                let mut base_img = GrayImage::from_raw(
                    raw_sprite.image.width(), raw_sprite.image.height(),
                    base
                ).expect("base image as u8s should fit in buffer of size w*h");
                // Create a kernel
                let kernel = kernel.of_size(size);
                let proc_kernel = Kernel::new(kernel.as_ref(), kernel.width(), kernel.height());
                // Convolve the base
                for _ in 1 .. level.abs() {
                    base_img = proc_kernel.filter(
                        &base_img, |_, _| ()
                    );
                }
                let offset = base_img.width() - raw_sprite.image.width();
                // Now that we've convolved, turn the base back to RGBA and apply
                let applied = RgbaImage::from_fn(base_img.width(), base_img.height(), |x, y| {
                    // Check the original pixel and apply it to the new image if needed
                    let px @ Rgba([_, _, _, alpha]) = raw_sprite.image.get_pixel(x, y);
                    if *alpha != 0 && (level % 2 == 1) && (level > 0) {
                        *px
                    } else if (*alpha != 0) ^ (level <= 0) {
                        Rgba([0; 4])
                    } else {
                        let Luma([px]) = base_img.get_pixel(x, y);
                        Rgba([*px; 4])
                    }
                });
                raw_sprite.image = applied;
            },
            others => new_variants.push(others)
        }
    }
    todo!()
}



type GrayImage = Image<Luma<u8>>;

impl MetaKernel {
    pub(crate) fn of_size(self, size: u8) -> GrayImage {
        let center = u32::from(size);
        let width = (center * 2) + 1;
        GrayImage::from_fn(width, width, |x, y| {
            let mut draw_pixel: bool = true;
            match self {
                // Sharp
                MetaKernel::Full =>
                    if x == center && y == center { draw_pixel = false },
                // Round
                MetaKernel::Edge => {
                    if x == center && y == center { draw_pixel = false }
                    if x == 0 && y == 0 { draw_pixel = false }
                    if x == 0 && y == width { draw_pixel = false }
                    if x == width && y == 0 { draw_pixel = false }
                    if x == width && y == width { draw_pixel = false }
                },
                // Round top, sharp bottom
                MetaKernel::Unit => {
                    if x == center && y == center { draw_pixel = false }
                    if x == 0 && y == 0 { draw_pixel = false }
                    if x == width && y == 0 { draw_pixel = false }
                },
            }
            if draw_pixel {Luma([u8::MAX])} else {Luma([0])}
        })
    }
}
