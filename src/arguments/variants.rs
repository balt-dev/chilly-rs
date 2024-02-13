//! Holds the variants supported by Chilly.

use std::fmt::Formatter;
use paste::paste;
pub use super::arguments::{
    MetaKernel,
    Argument,
    TilingDirection
};
use super::{RuntimeData, ArgumentError};
use crate::database::structures::Color;
use std::str::FromStr;

// I'm gonna be completely honest here.
// We never needed a proc macro for this.
// Time to do some macro_rules crimes.
macro_rules! variants {
    // Main implementation
    (
        variants = [$({
            $name: ident,
            [ $($alias: literal),+ ],
            $description: literal,
            [ $($argument: ty),* ]
        }),*],
        aliases = [$(
            $alias_name: ident : {
                $(($($aliased_value: tt)+) => $aliased_exp: expr),+
            }
        ),*]
    ) => {
        /// Holds runtime-accessible data about every variant supported by Chilly.
        pub static VARIANT_DATA: [RuntimeData<VariantName>; variants!(count $($name)*)] = [
            $(
                RuntimeData {
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
            /// Transforms a variant's alias into its canonical name, if there exists a variant with the argument as its alias.
            #[must_use]
            pub fn from_alias(alias: &str) -> Option<VariantName> {
                Some( match alias {
                    $($($alias)|+ => VariantName::$name,)+
                    _ => return None
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
            pub fn parse<'a>(name: VariantName, mut arguments: impl Iterator<Item = &'a str>) -> Result<Variant, ArgumentError> {
                Ok( match name {
                    $(VariantName::$name =>
                        {
                            variants!{parse $name arguments; $($argument),*; n; }
                        }
                    ),+
                } )
            }

            /// Collapses an aliased variant name directly into a variant, if it corresponds to one.
            #[must_use]
            #[allow(clippy::missing_panics_doc)]
            pub fn collapse_alias(alias: &str) -> Option<Variant> {
                Some ( match alias {
                    $($(
                        $($aliased_value)+ => $aliased_exp
                    ),+),+ ,
                    _ => return None
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
                return Err(ArgumentError::InvalidArgument(
                    "variant",
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
    variants = [
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
            AnimationFrame,
            ["frame", "f"],
            "Sets the animation frame of this tile.",
            [u8]
        },
        {
            Left,
            ["left", "l"],
            "Makes the tile face left if it supports directions.",
            []
        },
        {
            Up,
            ["up", "u"],
            "Makes the tile face up if it supports directions.",
            []
        },
        {
            Down,
            ["down", "d"],
            "Makes the tile face down if it supports directions.",
            []
        },
        {
            Right,
            ["right", "r"],
            "Makes the tile face right if it supports directions.",
            []
        },
        {
            Sleep,
            ["sleep", "s", "eepy"],
            "Puts the tile to sleep if it's a character tile.",
            []
        },
        {
            Animation,
            ["anim", "a"],
            "Set the tile's animation cycle.",
            [u8]
        },
        {
            Tiling,
            ["t", "tiling"],
            "Sets the tiling directions of this tile.",
            [Vec<TilingDirection>]
        },
        {
            Color,
            ["c", "color"],
            "Sets the color of the tile.\n\
             May be a palette index, a color name, or an RGB color.\n\
             This variant is aliased, so specifying the variant's name is optional.",
            [Color]
        },
        {
            Displace,
            ["disp", "d", "displace"],
            "Displaces a tile's position by a specified amount of pixels.",
            [isize, isize]
        }
    ],
    aliases = [
        Color: {
            // There is no way to bind to a match guard, so we parse twice :(
            (color_name if Color::from_str(color_name).is_ok()) =>
                Variant::Color(Color::from_str(color_name).expect("we checked that this works"))
        }
    ]
}
