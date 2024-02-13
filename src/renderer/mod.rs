#![cfg(feature = "rendering")]
//! Handles rendering of scenes into sprites.

use std::fmt::{Display, Formatter};
use std::path::Path;
use pest::error::ErrorVariant;
use pest::Span;
use thiserror::Error;
use crate::solidify::SkeletalScene;

#[derive(Debug, Clone, PartialEq, Error)]
/// Different things that can go wrong when rendering a scene.
pub enum RenderingError<'scene> {
    /// Couldn't find a sprite for a tile.
    NoSpriteFound(Span<'scene>)
}

impl<'scene> Display for RenderingError<'scene> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderingError::NoSpriteFound(span) => write!(
                f, "{}",
                pest::error::Error::<crate::parser::Rule>::new_from_span(
                    ErrorVariant::CustomError {
                        message: "\
                            no sprite files found for this tile\n\
                            this usually indicates missing files in the assets directory\
                        ".into()
                    },
                    *span
                )
            )
        }
    }
}

/// The main entrypoint in the renderer.
pub fn render<'db, 'scene>(scene: SkeletalScene<'db, 'scene>, sprite_path: impl AsRef<Path>) -> Result<RenderedScene, RenderingError<'scene>> {
    todo!("Rendering")
}
