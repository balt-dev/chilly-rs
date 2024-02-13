mod flags;
mod variants;
mod arguments;

pub use flags::*;
use thiserror::Error;
pub use variants::*;
pub use arguments::*;

use displaydoc::Display;

/// Holds runtime data about something.
/// Should mostly be used for a GUI.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeData<NAME> {
    /// The canonical name.
    pub name: NAME,
    /// A list of the aliases.
    pub aliases: &'static [&'static str],
    /// A description of what this does.
    pub description: &'static str,
    /// A list of types that this takes, as strings.
    pub arguments: &'static [&'static str]
}

/// Something went wrong while parsing an argument.
#[derive(Debug, Error, Display)]
pub enum ArgumentError {
    /// An argument of this was invalid.
    /// Supplies the original reason for the failure.
    #[displaydoc("argument {1} of the {0} was invalid: {2}")]
    InvalidArgument(&'static str, usize, Box<dyn std::error::Error>),
    /// The name for this argument does not exist.
    #[displaydoc("the {0} {1} does not exist")]
    NonExistentVariant(&'static str, String)
}