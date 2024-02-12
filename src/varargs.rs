//! Holds different classes and traits relating to variant arguments.

use std::str::FromStr;
use anyhow::anyhow;

mod sealed {
    use crate::varargs::MetaKernel;

    pub trait Sealed {}
    impl Sealed for MetaKernel {}
    impl Sealed for u8 {}
    impl Sealed for f32 {}
    impl<T: Sealed> Sealed for Option<T> {}
}

type BoxedErr = Box<dyn std::error::Error>;

/// A trait that dictates that this object is available for
/// parsing as a variant argument.
///
/// # Notes
/// - This trait is **not** object safe.
/// - This trait is [sealed.](https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed)
pub trait VariantArgument: sealed::Sealed + Sized {
    /// Parses values from the iterator until this type can be constructed.
    ///
    /// Returns None if failed.
    fn parse<'a>(args: impl Iterator<Item = &'a str>) -> Result<Self, BoxedErr>;
}


macro_rules! arg_unit_enum {
    ($name: ident : $($string: literal => $var: ident),+$(,)?) => {
        impl VariantArgument for $name {
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
        impl VariantArgument for $ty {
            fn parse<'a>(mut args: impl Iterator<Item=&'a str>) -> Result<Self, BoxedErr> {
                let arg = args.next().ok_or(
                    anyhow!("argument of type \"{}\" not supplied", stringify!($ty))
                )?;
                Ok(<$ty>::from_str(arg)?)
            }
        }
    )+ };
}

impl<T: VariantArgument + sealed::Sealed> VariantArgument for Option<T> {
    fn parse<'a>(mut args: impl Iterator<Item=&'a str>) -> Result<Self, BoxedErr> {
        let Some(arg) = args.next() else {
            return Ok(None)
        };
        T::parse([arg].into_iter()).map(Some)
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

arg_unit_enum!{
    MetaKernel:
        "full" => Full,
        "edge" => Edge,
        "unit" => Unit
}

arg_from_str! {
    u8 f32
}
