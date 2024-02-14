#![cfg(feature = "rendering")]
//! Handles rendering of scenes into sprites.

use std::{borrow::Cow, collections::HashMap, io, path::{Path, PathBuf}};
use crate::solidify::SkeletalScene;
use image::{io::Reader as ImageReader, Rgba, RgbaImage};

mod structures;
pub use structures::{
    RenderedScene, RenderingError
};
use crate::arguments::{Flag, FlagName};

/// Opens an image, potentially from a cache.
/// This will always return a borrow if given a cache, and always return something owned if not.
fn open_cached(
    path: impl AsRef<Path>,
    cache: Option<&mut HashMap<PathBuf, RgbaImage>>
) -> Result<Cow<RgbaImage>, io::Error> {
    let path = path.as_ref();
    if let Some(cached_image) = cache.and_then(|cache| cache.get(path)) {
        return Ok(Cow::Borrowed(cached_image));
    }
    let image = ImageReader::open(path)?
        .decode()
        .map_err(
            io::Error::other
        )?.into_rgba8();
    if let Some(cache) = cache {
        let cache_entry = cache.entry(path.to_path_buf()).or_insert(image);
        return Ok(Cow::Borrowed(cache_entry))
    }
    Ok(Cow::Owned(image))
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
pub fn render<'db, 'scene, 'cache>(
    mut scene: SkeletalScene<'db, 'scene>,
    asset_path: impl AsRef<Path>,
    cache: Option<&'cache mut HashMap<PathBuf, RgbaImage>>
) -> Result<RenderedScene<'cache>, RenderingError<'scene>> {
    let asset_path = asset_path.as_ref();

    // Parse boolean flags
    let loops = scene.flags.remove(&FlagName::NoLoop).is_none();

    // Get palette from assets
    let palette_path: PathBuf =
        if let Some(flag) = scene.flags.remove(&FlagName::Palette) {
            let Flag::Palette(path) = flag else {unreachable!()};
            path.into()
        } else { "default".into() };

    let palette = cache.as_ref().and_then(|cache| cache.get(&palette_path));
    // We use a match here to allow error propogation
    let palette = match palette {
        Some(v) => Cow::Borrowed(v),
        None => resolve_palette(asset_path.to_path_buf(), palette_path, cache)?
    };
    // Get background color
    let background_color = scene.flags.remove(&FlagName::BackgroundColor).and_then(|flag| {
        let Flag::BackgroundColor(color) = flag else {unreachable!()};
        color.into_rgba(palette.as_ref())
    }).unwrap_or(Rgba([0; 4]));
    todo!()
}

fn resolve_palette<'s, 'd>(
    asset_path: PathBuf,
    palette_path: PathBuf,
    cache: Option<&'d mut HashMap<PathBuf, RgbaImage>>
) -> Result<Cow<'d, RgbaImage>, RenderingError<'s>> {
    // Resolve the palette
    let mut glob_pattern = asset_path;
    glob_pattern.push("*");
    glob_pattern.push(palette_path.to_string_lossy().as_ref());
    let glob_pattern = glob_pattern.with_extension("png").to_string_lossy().to_string();
    let found_palette_path =
        glob::glob(&glob_pattern).expect("glob pattern was invalid")
            .filter_map(Result::ok)
            .next();
    let Some(path) = found_palette_path else {
        return Err(RenderingError::NoPalette(palette_path))
    };
    let mut reader = ImageReader::open(&path)
        .map_err(|err| RenderingError::FailedOpen(path.clone(), err))?;
    let image = reader.decode()
        .map_err(|err| RenderingError::FailedDecode(path, err))?
        .into_rgba8();
    if let Some(cache) = cache {
        Ok(Cow::Borrowed(cache.entry(palette_path).or_insert(image)))
    } else {
        Ok(Cow::Owned(image))
    }
}
