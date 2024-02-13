//! Holds different flags for Chilly.

use paste::paste;
use crate::database::structures::Color;

use super::{RuntimeData, ArgumentError, arguments::Argument};
use std::fmt::Formatter;

macro_rules! flags {
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
        pub static FLAG_DATA: [RuntimeData<FlagName>; flags!(count $($name)*)] = [
            $(
                RuntimeData {
                    name: FlagName::$name,
                    aliases: &[ $($alias),+ ],
                    description: $description,
                    arguments: &[ $(stringify!($argument)),* ]
                }
            ),*
        ];

        /// An enumeration over the canonical names of flags.
        #[non_exhaustive]
        #[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
        #[allow(missing_docs)]
        pub enum FlagName {
            $($name),+
        }

        impl std::fmt::Display for FlagName {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", match self {
                    $(Self::$name => stringify!($name)),+
                })
            }
        }

        impl FlagName {
            /// Transforms a flag's alias into its canonical name, if there exists a flag with the argument as its alias.
            #[must_use]
            pub fn from_alias(alias: &str) -> Option<FlagName> {
                Some( match alias {
                    $($($alias)|+ => FlagName::$name,)+
                    _ => return None
                } )
            }
        }

        /// Holds an enumeration over every flag supported by Chilly.
        #[non_exhaustive]
        #[derive(Debug, PartialEq, Eq, Hash, Clone)]
        pub enum Flag {
            $(
                #[doc = $description]
                $name ( $($argument),* )
            ),*
        }

        impl Flag {

            /// Parses a flag from its canonical name and a list of arguments.
            ///
            /// # Errors
            /// Errors if a flag argument fails to parse.
            pub fn parse<'a>(name: FlagName, mut arguments: impl Iterator<Item = &'a str>) -> Result<Flag, ArgumentError> {
                Ok( match name {
                    $(FlagName::$name =>
                        {
                            flags!{parse $name arguments; $($argument),*; n; }
                        }
                    ),+
                } )
            }
        }
    };
    // Hold a counter of tokens for help with the FLAG_DATA slice
    (count $tt: tt $($tts: tt)*) => {
        1 + flags!(count $($tts)*)
    };
    // Base case
    (count) => {0};
    // Parse a single variant argument, moving on to the next
    (parse $name: ident $args: ident; $ty: ty $(, $($tys: ty),+)?; $count: ident; $($argname: ident)* ) => { paste! {

        let [< arg_ $count >] = match <$ty>::parse(&mut $args) {
            Ok(v) => v,
            Err(err) => {
                return Err(ArgumentError::InvalidArgument(
                    "flag",
                    flags!{count $($argname)*},
                    err
                ))
            }
        };

        flags!{ parse $name $args; $($($tys),+)?; [< $count n >]; $($argname)* [< arg_ $count >] }
    } };
    // Base case, construct the flag from the identifiers we parsed from
    (parse $name: ident $args: ident; ; $_: ident; $($argname: ident)*) => {
        Flag::$name (
            $($argname),*
        )
    }
}

flags! {
    {
        BackgroundColor,
        ["b", "background"],
        "Sets the background color of this scene.",
        [Color]
    },
    {
        ConnectBorders,
        ["tb", "tile_borders"],
        "Connects any autotiling tiles to the borders of the scene.",
        []
    },
    {
        UseLetters,
        ["letters", "let"],
        "Defaults to using letters for text generation.",
        []
    }
}

