use std::{
    fmt::{Display, Formatter},
    io,
    collections::HashMap
};
use std::borrow::Cow;
use std::path::PathBuf;
use std::time::Duration;

use image::{ImageError, Rgba, RgbaImage};
use pest::{error::ErrorVariant, Span};
use thiserror::Error;
use crate::{arguments::{Flag, FlagName}, database::structures::Color};


/// A rendered scene, ready to be passed back to the renderer implementation.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderedScene<'cache> {
    /// The background color of the scene.
    pub background: Rgba<u8>,
    /// The flags to pass back to the implementation.
    pub flags: HashMap<FlagName, Flag>,
    /// The pixel width of the rendered scene.
    pub width: usize,
    /// The pixel height of the rendered scene.
    pub height: usize,
    /// A list of different frames in the scene.
    pub frames: Vec<SceneFrame<'cache>>,
    /// Whether the scene should loop.
    pub loops: bool
}

/// A single frame of a rendered scene.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneFrame<'cache> {
    /// The length of this frame.
    pub length: Duration,
    /// The separate sprites in this frame.
    pub sprites: Vec<Sprite<'cache>>
}

/// A single sprite in a rendered scene.
#[derive(Debug, Clone, PartialEq)]
pub struct Sprite<'cache> {
    /// The size multiplier that this sprite has.
    pub size: f32,
    /// The [Z order](https://en.wikipedia.org/wiki/Z-order) of this sprite in the given frame.
    ///
    /// This is an opaque value that's only meant to give an order to draw sprites in.
    /// Don't depend on its actual value.
    pub z_order: usize,
    /// The pixel position of this sprite in the frame.
    pub position: (isize, isize),
    /// The sprite's image.
    pub image: Cow<'cache, RgbaImage>
}

#[derive(Debug, Error)]
/// Different things that can go wrong when rendering a scene.
pub enum RenderingError<'scene> {
    /// Failed to open a sprite for a tile.
    SpriteFailedOpen(Span<'scene>, io::Error),
    /// The given tile doesn't exist.
    SpriteNoTile(Span<'scene>, String),
    /// Couldn't find a palette.
    SpriteNoPalette(Span<'scene>, PathBuf),
    /// Failed to decode an image.
    SpriteFailedDecode(Span<'scene>, PathBuf, ImageError),
    /// Couldn't find a palette for the scene.
    NoPalette(PathBuf),
    /// Failed to open something that isn't a sprite.
    FailedOpen(PathBuf, io::Error),
    /// Failed to decode an image.
    FailedDecode(PathBuf, ImageError),
}

macro_rules! spanned_err {
    ($f: ident, $span: ident, $($message: tt)+) => {
        write!(
            $f, "{}",
            pest::error::Error::<crate::parser::Rule>::new_from_span(
                ErrorVariant::CustomError {
                    message: format!($($message)+)
                },
                *$span
            )
        )
    }
}

impl<'scene> Display for RenderingError<'scene> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderingError::SpriteFailedOpen(span, err) => 
                spanned_err!(
                    f, span, 
                    "couldn't open a sprite for this tile\n\
                     error: {err}"
                ),
            RenderingError::SpriteNoTile(span, err) => 
            spanned_err!(
                f, span, 
                "couldn't open a sprite for this tile\n\
                 error: {err}"
            ),
            RenderingError::SpriteNoPalette(_, _) => todo!(),
            RenderingError::SpriteFailedDecode(_, _, _) => todo!(),
            RenderingError::NoPalette(path) => write!(
                f, "couldn't find a palette named {}", path.display()
            ),
            RenderingError::FailedOpen(path, err) =>
                write!(f, "failed to open \"{}\": {err}", path.display()),
            RenderingError::FailedDecode(path, err) =>
                write!(f, "failed to decode image at \"{}\": {err}", path.display()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RawSprite<'cache> {
    pub(crate) image: Cow<'cache, RgbaImage>,
    pub(crate) color: Color
}