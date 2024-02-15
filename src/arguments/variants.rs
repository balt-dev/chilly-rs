//! Holds the variants supported by Chilly.

use std::fmt::Formatter;
use paste::paste;
pub use super::args::{
    MetaKernel,
    Argument,
    TilingDirection
};
use super::{RuntimeData, ArgumentError, arg_macro};
use crate::database::structures::Color;
use std::str::FromStr;

arg_macro! {
    data_name: VARIANT_DATA,
    data_kind: Variant,
    variants: [
        {
            Meta,
            ["meta", "m"],
           "Adds an outline to a tile's sprite.\n\
            Optionally, it can be specified how many times to outline, \
            an outline kernel to use, and an outline size.",
            [Option<i8>, Option<MetaKernel>, Option<u8>]
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
            ["disp", "displace"],
            "Displaces a tile's position by a specified amount of pixels.",
            [isize, isize]
        }
    ],
    aliases: [
        Color: {
            // There is no way to bind to a match guard, so we parse twice :(
            (color_name if Color::from_str(color_name).is_ok()) =>
                Variant::Color(Color::from_str(color_name).expect("we checked that this works"))
        }
    ]
}
