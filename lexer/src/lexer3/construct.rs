use super::*;
use lexer3_macros::Spanable;
use std::str::CharIndices;

macro_rules! fast {
    () => {};
    ($a:ident $reg:literal $($t:tt)*) => {
        paste!{
            pub struct [<$a Marker>];

            impl RegularToken for [<$a Marker>] {
                const REG_EXPR: &'static str = $reg;
            }

            pub type $a = Token<[<$a Marker>]>;
        }
        fast!($($t)*);
    };
    ($name:ident { $( $a:ident )|+ } $($t:tt)*) => {
        pub enum $name {
            $($a($a)),+
        }

        impl EnumRecog for $name {
            type Output = Self;
            const N: usize = ${count($a)};

            fn structure_assembling<'a>(
                code: &'a Code,
            ) -> [Box<dyn core::ops::Fn() -> Result<Self::Output, &'static str> + 'a>; Self::N] {
                [
                    $(
                        Box::new(|| $a::recog(code).map(Self::$a))
                    ),+
                ]
            }
        }
        fast!($($t)*);
    };
    ($name:ident ( $( $a:ident )+ ) $($t:tt)*) => {
        pub type $name = ( $($a),+ );
        fast!($($t)*);
    };
    ($name:ident $item:ident $ignore:ident $break:ident $($t:tt)*) => {
        fast!($($t)*);
    };
}

fast!(
    ImplItemsI ImplItemsV Ignore CloseFigureBracket
    Op { Add | Sub | Mul | Div }
        Add r#"\+"#
        Sub r#"-"#
        Mul r"\*"
        Div r#"/"#
    // keywords
    Fn "fn"
    Const "const"
    Struct "struct"
    Trait "trait"
    Let "let"
    Impl "impl"
    Crate "crate"
    For "for"
    SelfT "self"
    Super "super"

    // other
    Number r"\b\d+\b"

    Distribution r#"\.\."#
    NameSpace "::"
    OpenFigureBracket r#"\{"#
    CloseFigureBracket r#"}"#
    OpenRoundBracket r#"\("#
    CloseRoundBracket r#"\)"#
    Eq r#"="#

    Comma ","
    Colon ":"
);
