//! Holds different flags for Chilly.

use paste::paste;
use crate::database::structures::Color;

use super::{RuntimeData, ArgumentError, args::Argument, arg_macro};
use std::fmt::Formatter;


arg_macro! {
    data_name: FLAG_DATA,
    data_kind: Flag,
    variants: [
        {
            BackgroundColor,
            ["b", "background"],
            "Sets the background color of this scene.",
            [Option<Color>]
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
        },
        {
            Palette,
            ["p", "pal", "palette"],
            "Specifies a palette to use for a scene.",
            [String]
        },
        {
            NoLoop,
            ["nl", "noloop"],
            "Prevents the rendered scene from looping.",
            []
        },
        {
            DecoupleWobble,
            ["am", "anim"],
            "\
                Decouples the wobble frames from the animation frames, putting them into a polyrhythmic relation.\n\
                The first argument is the number of render frames to wait for the next wobble frame,\
                and the second is the number of render frames to wait for the next animation frame.
            ",
            [u8, u8]
        }
    ],
    aliases: []
}

