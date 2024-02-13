use std::{error::Error, fmt::{Display, Formatter}, io};

use image::Rgba;
use pest::{error::ErrorVariant, Span};
use thiserror::Error;


/// A rendered scene, ready to be passed back to the renderer implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedScene {
    /// The background color of the scene.
    pub background: Rgba<u8>,
    
}

#[derive(Debug, Error)]
/// Different things that can go wrong when rendering a scene.
pub enum RenderingError<'scene> {
    /// Failed to open a sprite for a tile.
    SpriteFailedOpen(Span<'scene>, io::Error)
}

impl<'scene> Display for RenderingError<'scene> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderingError::SpriteFailedOpen(span, err) => write!(
                f, "{}",
                pest::error::Error::<crate::parser::Rule>::new_from_span(
                    ErrorVariant::CustomError {
                        message: format!("\
                            failed to open sprite files for this tile\n\
                            error: {err}\
                        ")
                    },
                    *span
                )
            )
        }
    }
}