#![cfg(feature = "rendering")]
//! Handles rendering of scenes into sprites.

use std::{
    borrow::Cow, 
    collections::HashMap, 
    fmt::{Display, Formatter}, 
    fs::File, 
    path::{Path, PathBuf},
    io
};
use pest::{
    error::ErrorVariant, Span
};
use thiserror::Error;
use crate::solidify::SkeletalScene;
use image::{io::Reader as ImageReader, Rgba, RgbaImage};

mod structures;
pub use structures::{
    RenderedScene, RenderingError
};

/// Opens an image, potentially from a cache.
fn open_cached<'cache>(
    path: impl AsRef<Path>,
    cache: Option<&'cache mut HashMap<PathBuf, RgbaImage>>
) -> Result<Cow<'cache, RgbaImage>, std::io::Error> {
    let path = path.as_ref();
    if let Some(cache) = cache {
        if let Some(cached_image) = cache.get(path) {
            return Ok(Cow::Borrowed(cached_image));
        }
    }
    let image = ImageReader::open(path)?
    .decode()
    .map_err(
        std::io::Error::other
    )?.into_rgba8();
    if let Some(cache) = cache {
        cache.insert(path.to_path_buf(), image);
        let cache_entry = cache.get(path).unwrap();
        return Ok(Cow::Borrowed(cache_entry))
    }
    Ok(Cow::Owned(image))
}

impl<'db, 'scene> SkeletalScene<'db, 'scene> {
    /// The main entrypoint in the renderer.
    /// 
    /// If a cache can be passed in, then paths
    /// from the cache will be used instead of
    /// opening the file at the path. 
    pub fn render<'cache>(
        mut self: SkeletalScene<'db, 'scene>,
        sprite_path: impl AsRef<Path>,
        palette_path: impl AsRef<Path>,
        cache: Option<&'cache mut HashMap<PathBuf, RgbaImage>>
    ) -> Result<RenderedScene, RenderingError<'scene>> {
        // Parse the flags if they're there
        let mut background = Rgba::from([0u8; 4]);
        if let Some(flag) = self.flags.remove("b")
            .or(self.flags.remove("background")) 
        {
            
        }
    }
}
