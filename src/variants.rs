//! Holds the variants supported by Chilly.

use std::fmt::Formatter;
use displaydoc::Display;
use paste::paste;
use thiserror::Error;
use crate::varargs::{
    MetaKernel,
    VariantArgument,
};

/// Something went wrong while parsing a variant.
#[derive(Debug, Error, Display)]
pub enum VariantError {
    /// An argument of this variant was invalid.
    /// Supplies the original reason for the failure.
    #[displaydoc("Argument {0} of this variant was invalid: {1}")]
    InvalidArgument(usize, Box<dyn std::error::Error>),
    /// The variant name does not exist.
    #[displaydoc("The variant {0} does not exist.")]
    NonExistentVariant(String)
}

// I'm gonna be completely honest here.
// We never needed a proc macro for this.
// Time to do some macro_rules crimes.
macro_rules! variants {
    // Main implementation
    (
        $({
            $name: ident,
            [ $($alias: literal),+ ],
            $description: literal,
            [ $($argument: ty),* ]
        }),*
    ) => {
        /// Holds runtime-accessible data about every variant supported by Chilly.
        pub static VARIANT_DATA: [VariantData; variants!(count $($name)*)] = [
            $(
                VariantData {
                    name: VariantName::$name,
                    aliases: &[ $($alias),+ ],
                    description: $description,
                    arguments: &[ $(stringify!($argument)),* ]
                }
            ),*
        ];

        /// An enumeration over the canonical names of variants.
        #[non_exhaustive]
        #[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
        #[allow(missing_docs)]
        pub enum VariantName {
            $($name),+
        }

        impl VariantName {
            /// Transforms a variant's alias into its canonical name.
            ///
            /// # Errors
            /// Errors with [`VariantError::NonExistentVariant`] if no variant with this alias exists.
            #[must_use]
            pub fn from_alias(alias: &str) -> Result<VariantName, VariantError> {
                Ok( match alias {
                    $($($alias)|+ => VariantName::$name,)+
                    _ => return Err(VariantError::NonExistentVariant(alias.to_string()))
                } )
            }
        }

        impl std::fmt::Display for VariantName {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", match self {
                    $(Self::$name => stringify!($name)),+
                })
            }
        }

        /// Holds an enumeration over every variant supported by Chilly.
        #[non_exhaustive]
        #[derive(Debug, PartialEq, Clone)]
        pub enum Variant {
            $(
                #[doc = $description]
                $name ( $($argument),* )
            ),*
        }

        impl Variant {

            /// Parses a variant from its canonical name and a list of arguments.
            ///
            /// # Errors
            /// Errors if a variant argument fails to parse.
            pub fn parse<'a>(name: VariantName, mut arguments: impl Iterator<Item = &'a str>) -> Result<Variant, VariantError> {
                Ok( match name {
                    $(VariantName::$name =>
                        {
                            variants!{parse $name arguments; $($argument),*; n; }
                        }
                    ),+
                } )
            }
        }
    };
    // Hold a counter of tokens for help with the VARIANT_DATA slice
    (count $tt: tt $($tts: tt)*) => {
        1 + variants!(count $($tts)*)
    };
    // Base case
    (count) => {0};
    // Parse a single variant argument, moving on to the next
    (parse $name: ident $args: ident; $ty: ty $(, $($tys: ty),+)?; $count: ident; $($argname: ident)* ) => { paste! {

        let [< arg_ $count >] = match <$ty>::parse(&mut $args) {
            Ok(v) => v,
            Err(err) => {
                return Err(VariantError::InvalidArgument(
                    variants!{count $($argname)*},
                    err
                ))
            }
        };

        variants!{ parse $name $args; $($($tys),+)?; [< $count n >]; $($argname)* [< arg_ $count >] }
    } };
    // Base case, construct the variant from the identifiers we parsed from
    (parse $name: ident $args: ident; ; $_: ident; $($argname: ident)*) => {
        Variant::$name (
            $($argname),*
        )
    }
}

variants! {
    {
        Meta,
        ["meta", "m"],
       "Adds an outline to a tile's sprite.\n\
        Optionally, it can be specified how many times to outline, \
        an outline kernel to use, and an outline size.",
        [Option<u8>, Option<MetaKernel>, Option<u8>]
    },
    {
        Noop,
        [""],
        "Does nothing. Useful for resetting variants on animations.",
        []
    },
    {
        Argless,
        ["test"],
        "No argument test",
        []
    },
    {
        MandArgs,
        ["mand"],
        "Mandatory args",
        [u8, Option<u8>]
    }
}

/// Holds the data behind one variant.
/// Should mostly be used for runtime documentation in a GUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VariantData {
    name: VariantName,
    aliases: &'static [&'static str],
    description: &'static str,
    arguments: &'static [&'static str]
}
