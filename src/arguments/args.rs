//! Holds different classes and traits relating to variant arguments.

use std::str::FromStr;
use anyhow::anyhow;
use crate::database::structures::Color;

mod sealed {
    use crate::database::structures::Color;
    use super::{MetaKernel, TilingDirection};

    pub trait Sealed {}
    impl Sealed for MetaKernel {}
    impl Sealed for TilingDirection {}
    impl Sealed for u8 {}
    impl Sealed for isize {}
    impl Sealed for f32 {}
    impl Sealed for Color {}
    impl Sealed for String {}
    impl<T: Sealed> Sealed for Option<T> {}
    impl<T: Sealed> Sealed for Vec<T> {}
    impl<const N: usize, T: Sealed> Sealed for [T; N] {}
}

type BoxedErr = Box<dyn std::error::Error>;

/// A trait that dictates that this object is available for
/// parsing as a variant argument.
///
/// # Notes
/// - This trait is **not** object safe.
/// - This trait is [sealed.](https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed)
pub trait Argument: sealed::Sealed + Sized {
    /// Parses values from the iterator until this type can be constructed.
    ///
    /// # Errors
    /// Returns a [`Box<dyn std::error::Error>`] when an argument fails to parse.
    ///
    /// A type can fail to parse for any number of reasons, so the error is left generic.
    fn parse<'a>(args: impl Iterator<Item = &'a str>) -> Result<Self, BoxedErr>;
}


macro_rules! arg_unit_enum {
    ($name: ident : $($string: literal => $var: ident),+$(,)?) => {
        impl Argument for $name {
            fn parse<'a>(mut args: impl Iterator<Item=&'a str>) -> Result<Self, BoxedErr> {
                let arg = args.next().ok_or(
                    anyhow!("argument of type \"{}\" not supplied", stringify!($name))
                )?;
                Ok( match arg {
                    $($string => Self::$var,)+
                    _ => return Err(
                        anyhow!("must be one of: {}", [$($string),+].join(", ")).into()
                    )
                } )
            }
        }
    };
}

macro_rules! arg_from_str {
    ($($ty: ty)+) => { $(
        impl Argument for $ty {
            fn parse<'a>(mut args: impl Iterator<Item=&'a str>) -> Result<Self, BoxedErr> {
                let arg = args.next().ok_or(
                    anyhow!("argument of type \"{}\" not supplied", stringify!($ty))
                )?;
                Ok(<$ty>::from_str(arg)?)
            }
        }
    )+ };
}

impl<T: Argument + sealed::Sealed> Argument for Option<T> {
    fn parse<'a>(mut args: impl Iterator<Item=&'a str>) -> Result<Self, BoxedErr> {
        let Some(arg) = args.next() else {
            return Ok(None)
        };
        T::parse([arg].into_iter()).map(Some)
    }
}

impl<T: Argument + sealed::Sealed> Argument for Vec<T> {
    fn parse<'a>(args: impl Iterator<Item=&'a str>) -> Result<Self, BoxedErr> {
        args.map(
            |arg| T::parse([arg].into_iter())
        ).collect()
    }
}

impl<const N: usize, T: Argument + sealed::Sealed> Argument for [T; N] {
    fn parse<'a>(args: impl Iterator<Item=&'a str>) -> Result<Self, BoxedErr> {
        // TODO: When https://github.com/rust-lang/rust/issues/89379 is stabilized, this can be optimized
        let args = args.take(N).map(
            |arg| T::parse([arg].into_iter())
        ).collect::<Result<Vec<_>, _>>()?;
        let len = args.len();
        let args: [T; N] = args.try_into().map_err(
            |_| anyhow!("wrong amount of arguments for array of size {N} (got {len})")
        )?;
        Ok(args)
    }
}

/// A kernel to use for the [`Variant::Meta`] effect.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub enum MetaKernel {
    /// Uses a kernel that gives sharp corners.
    #[default]
    Full,
    /// Uses a kernel that gives round corners.
    Edge,
    /// Uses a kernel that gives sharp corners on top,
    /// but round corners on bottom.
    Unit
}

/// A tiling direction for a tile to connect to. Used in [`Variant::Tiling`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum TilingDirection {
    Right, UpRight, Up, UpLeft, Left, DownLeft, Down, DownRight
}

arg_unit_enum!{
    MetaKernel:
        "full" => Full,
        "edge" => Edge,
        "unit" => Unit
}

arg_unit_enum!{
    TilingDirection:
        "r" => Right,
        "u" => Up,
        "l" => Left,
        "d" => Down,
        "ur" => UpRight,
        "ul" => UpLeft,
        "dl" => DownLeft,
        "dr" => DownRight
}

arg_from_str! {
    u8 f32 isize Color String
}
