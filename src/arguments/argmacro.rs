//! Holds a macro for convenient parsed things.
//! Used for both flags and variants.

// I'm gonna be completely honest here.
// We never needed a proc macro for this.
// Time to do some macro_rules crimes.
macro_rules! arg_macro {
    // Main implementation
    (
        data_name: $dataname: ident,
        data_kind: $datakind: ident,
        variants: [$({
            $name: ident,
            [ $($alias: literal),+ ],
            $description: literal,
            [ $($argument: ty),* ]
        }),*],
        aliases: [$(
            $alias_name: ident : {
                $(($($aliased_value: tt)+) => $aliased_exp: expr),+
            }
        ),*]
    ) => { paste! {
        /// Holds runtime-accessible data about every variant supported by Chilly.
        pub static $dataname: [RuntimeData<[< $datakind Name >]>; arg_macro!(count $($name)*)] = [
            $(
                RuntimeData {
                    name: [< $datakind Name >]::$name,
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
        pub enum [< $datakind Name >] {
            $($name),+
        }

        impl [< $datakind Name >] {
            #[doc = concat!(
                "Transforms an alias for a ", stringify!($datakind), 
                " into its canonical name, if it corresponds to one."
            )]            #[must_use]
            pub fn from_alias(alias: &str) -> Option<[< $datakind Name >]> {
                Some( match alias {
                    $($($alias)|+ => [< $datakind Name >]::$name,)+
                    _ => return None
                } )
            }
        }

        impl std::fmt::Display for [< $datakind Name >] {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", match self {
                    $(Self::$name => stringify!($name)),+
                })
            }
        }

        #[doc = concat!("Holds an enumeration over every ", stringify!($datakind), " supported by Chilly.")]
        #[non_exhaustive]
        #[derive(Debug, PartialEq, Clone)]
        pub enum $datakind {
            $(
                #[doc = $description]
                $name ( $($argument),* )
            ),*
        }

        impl $datakind {

            #[doc = concat!("Parses a ", stringify!($datakind), " from its canonical name and a list of arguments.")]
            ///
            /// # Errors
            #[doc = concat!("Errors if a ", stringify!($datakind), "argument fails to parse.")]
            pub fn parse<'a>(name: [< $datakind Name >], mut arguments: impl Iterator<Item = &'a str>) -> Result<$datakind, ArgumentError> {
                Ok( match name {
                    $([< $datakind Name >]::$name =>
                        {
                            arg_macro!{parse $datakind, $name arguments; $($argument),*; n; }
                        }
                    ),+
                } )
            }

            #[doc = concat!(
                "Collapses an aliased ", stringify!($datakind), 
                "name directly into a ", stringify!($datakind), 
                ", if it corresponds to one."
            )]
            #[must_use]
            #[allow(clippy::missing_panics_doc, unreachable_code)]
            pub fn collapse_alias(alias: &str) -> Option<$datakind> {
                Some ( match alias {
                    $($(
                        $($aliased_value)+ => $aliased_exp
                    ),+,)*
                    _ => return None
                } )
            }
        }
    } };
    // Hold a counter of tokens for help with the data slice
    (count $tt: tt $($tts: tt)*) => {
        1 + arg_macro!(count $($tts)*)
    };
    // Base case
    (count) => {0};
    // Parse a single argument, moving on to the next
    (parse $datakind: ident, $name: ident $args: ident; $ty: ty $(, $($tys: ty),+)?; $count: ident; $($argname: ident)* ) => { paste! {

        let [< arg_ $count >] = match <$ty>::parse(&mut $args) {
            Ok(v) => v,
            Err(err) => {
                return Err(ArgumentError::InvalidArgument(
                    stringify!($datakind),
                    arg_macro!{count $($argname)*},
                    err
                ))
            }
        };

        arg_macro!{ parse $datakind, $name $args; $($($tys),+)?; [< $count n >]; $($argname)* [< arg_ $count >] }
    } };
    // Base case, construct from the identifiers we parsed from
    (parse $datakind: ident, $name: ident $args: ident; ; $_: ident; $($argname: ident)*) => {
        $datakind::$name (
            $($argname),*
        )
    }
}

pub(crate) use arg_macro;