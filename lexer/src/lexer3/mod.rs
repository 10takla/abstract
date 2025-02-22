use paste::paste;
use regex::Regex;
use std::{any::Any, marker::PhantomData, ops::Range};

type Slice = Range<usize>;

pub trait Spanable {
    fn span(&self) -> Slice;
}

trait CommonRecog {
    fn recog(code: &Code) -> Result<Self, &'static str>
    where
        Self: Sized;
}

mod token {
    use super::*;

    #[derive(Debug, PartialEq, Clone)]
    pub struct Token<T> {
        pub span: Slice,
        pub _marker: PhantomData<T>,
    }

    impl<T> Token<T> {
        pub const fn new(span: Slice) -> Self {
            Self {
                span,
                _marker: PhantomData,
            }
        }
    }

    pub trait RegularToken {
        const REG_EXPR: &'static str;
    }

    pub trait TokenRecog {
        type Output;
        // распознавание относительно курсора, то есть с учетом смещения строки, без продвижения
        fn cursor_aware_recog(code: &Code) -> Result<Token<Self::Output>, &'static str> {
            Self::start_string_aware_recog(&code.source[code.cursor..]).map(|span| Token {
                _marker: PhantomData,
                span: code.cursor + span.start..code.cursor + span.end,
            })
        }

        // расознавания происходит относительно строки, без каких либо смещений по курсору
        fn start_string_aware_recog(code: &str) -> Result<Slice, &'static str>;
    }

    impl<T: RegularToken> TokenRecog for Token<T> {
        type Output = T;
        fn start_string_aware_recog(code: &str) -> Result<Slice, &'static str> {
            Regex::new(&format!("^{}", T::REG_EXPR))
                .unwrap()
                .find(code)
                .map(|mat| mat.range())
                .ok_or("Не совпала с регуляркой")
        }
    }

    impl<T> Spanable for Token<T> {
        fn span(&self) -> Slice {
            self.span.clone()
        }
    }

    impl<T> CommonRecog for Token<T>
    where
        Token<T>: TokenRecog<Output = T>,
    {
        fn recog(code: &Code) -> Result<Self, &'static str> {
            Token::<T>::cursor_aware_recog(code)
        }
    }
}

use token::*;

mod sequence {
    use super::*;

    pub trait SequenceRecog: Sized {
        // распознавнаие с продвижением курсора
        fn cursor_aware_recog(code: &Code) -> Result<Self, &'static str> {
            Self::structure_assembling(&mut code.clone())
        }

        fn structure_assembling(code: &mut Code) -> Result<Self, &'static str>;

        fn promotion<T: CommonRecog + Spanable>(code: &mut Code) -> Result<T, &'static str> {
            T::recog(code).map(|v| {
                code.cursor = v.span().end;
                v
            })
        }
    }

    impl<T: SequenceRecog> CommonRecog for T {
        fn recog(code: &Code) -> Result<Self, &'static str> {
            Self::cursor_aware_recog(code)
        }
    }

    #[macro_export]
    macro_rules! tuple_impl {
        ($macros:ident! $(@$key:ident)?) => {
            tuple_impl!(@recursive $macros! $(@$key)? T T T T T T T T T T T T T T T T T T T T T T T T);
        };
        (@recursive $macros:ident! $(@$key:ident)? $a:ident) => {};
        (@recursive $macros:ident! $(@$key:ident)? $a:ident $($other:ident)+) => {
            tuple_impl!(@recursive $macros! $(@$key)? $($other)+);
            tuple_impl!(@named $macros! $(@$key)? $a $($other)+);
        };

        (@named $macros:ident! $(@$key:ident)? $($a:ident)+) => {
            paste!(
                $macros!($(@$key)? $([<$a ${index()}>])+);
            );
        };
    }

    macro_rules! impl_seq {
        (@impl $($a:ident)+) => {
            impl_seq!(@spanable $($a)+);
            impl<$($a: CommonRecog + Spanable),+> SequenceRecog for ($($a),+) {
                fn structure_assembling(code: &mut Code) -> Result<Self, &'static str> {
                    Ok(($(Self::promotion::<$a>(code)?),+))
                }
            }
        };
        (@spanable $a:ident $($other:ident)+ ) => {
            impl<$a: Spanable, $($other: Spanable),+> Spanable for ($a, $($other),+) {
                fn span(&self) -> Slice {
                    self.0.span().start..self.${count($other)}.span().end
                }
            }
        };
    }
    tuple_impl!(impl_seq! @impl);
}
use sequence::*;

mod dyn_sequence {
    use super::*;
    use crate::tuple_impl;

    type DynType = Box<dyn DynSpanable>;
    pub trait DynSpanable: Spanable + Any {}

    impl dyn DynSpanable {
        fn as_any(&self) -> &dyn Any {
            self as &dyn Any
        }
    }

    impl<T: Spanable + Any> DynSpanable for T {}

    pub trait DynSeqRecog {
        type Output;
        fn promotion_recog(self, code: &Code) -> Result<Self::Output, &'static str>;
        fn promotion(v: Box<dyn DynRecog>, code: &mut Code) -> Result<DynType, &'static str>;
        fn structure_assembling(self, code: &mut Code) -> Result<Self::Output, &'static str>;
    }

    impl<const N: usize> DynSeqRecog for [Box<dyn DynRecog>; N] {
        type Output = [DynType; N];
        fn promotion_recog(self, code: &Code) -> Result<Self::Output, &'static str> {
            self.structure_assembling(&mut code.clone())
        }

        fn structure_assembling(self, code: &mut Code) -> Result<Self::Output, &'static str> {
            self.try_map(|v| Self::promotion(v, code))
        }

        fn promotion(v: Box<dyn DynRecog>, code: &mut Code) -> Result<DynType, &'static str> {
            v.recog(code).map(|v| {
                code.cursor = v.span().end;
                v
            })
        }
    }

    pub trait DynRecog {
        fn recog(&self, code: &mut Code) -> Result<DynType, &'static str>;
    }

    impl<T: MarkerConversion + 'static> DynRecog for T {
        fn recog(&self, code: &mut Code) -> Result<DynType, &'static str> {
            T::conv(code).map(|v| Box::new(v) as DynType)
        }
    }

    /// посрденик к типовому распознаванию, так как `[Box<dyn _>; _]` требует `self`
    pub trait MarkerConversion {
        type Output: CommonRecog + Spanable;
        fn conv(code: &mut Code) -> Result<Self::Output, &'static str> {
            Self::Output::recog(code)
        }
    }

    /// для токенов
    impl<T> MarkerConversion for T
    where
        Token<T>: CommonRecog,
    {
        type Output = Token<T>;
    }

    // для последовательностей
    macro_rules! dyn_impl_seq {
        (@impl $($a:ident)+) => {
            impl<$($a: MarkerConversion),+> MarkerConversion for ($($a),+) {
                type Output = ($($a::Output),+);
            }
        };
    }

    tuple_impl!(dyn_impl_seq! @impl);

    use super::{items::*, *};

    #[test]
    fn test() {
        macro_rules! check {
            (($code:literal, [$($a:expr),+], [$($c:ty),+]), $b:expr) => {
                let v = [$($a as Box<dyn DynRecog>),+].promotion_recog(&$code.into()).unwrap();

                $(
                    assert_eq!(
                        *v[${index()}].as_any().downcast_ref::<$c>().unwrap(),
                        $b[${index()}]
                    );
                )+
            };
        }

        check!(
            (r#"tmp"n""#, [Box::new(Ident), Box::new(String)], [Token<Ident>, Token<String>]),
            [Token::new(0..3), Token::new(3..6)]
        );
        check!(
            (r#"tmp"n"tmp"#, [Box::new(IdentStringMarker)], [IdentString]),
            [IdentString(
                Token::new(0..3),
                Token::new(3..6),
                Token::new(6..9)
            )]
        );
        check!(
            (r#"tmp"n""#, [Box::new((Ident, String))], [(Token<Ident>, Token<String>)]),
            [(Token::new(0..3), Token::new(3..6))]
        );
    }
}
use dyn_sequence::*;

mod enum_ {
    use super::*;
    use crate::tuple_impl;

    pub trait EnumRecog {
        type Output;
        const N: usize;
        fn cursor_aware_recog(code: &Code) -> Result<Self::Output, Vec<&'static str>>
        where
            [(); Self::N]: Sized,
        {
            let mut errs = vec![];
            Self::structure_assembling(code)
                .into_iter()
                .find_map(|v| v().map_err(|e| errs.push(e)).ok())
                .ok_or(errs)
        }
        fn structure_assembling<'a>(
            code: &'a Code,
        ) -> [Box<dyn core::ops::Fn() -> Result<Self::Output, &'static str> + 'a>; Self::N];
    }

    impl<T: EnumRecog> CommonRecog for Enum<T::Output, T>
    where
        [(); T::N]: Sized,
    {
        fn recog(code: &Code) -> Result<Self, &'static str>
        where
            Self: Sized,
        {
            T::cursor_aware_recog(code)
                .map_err(|v| v[0])
                .map(|v| Self(v, PhantomData))
        }
    }

    #[derive(Debug, PartialEq)]
    struct Enum<Output, T = Output>(Output, PhantomData<T>);
    impl<Output, T> Enum<Output, T> {
        pub const fn new(inner: Output) -> Self {
            Self(inner, PhantomData)
        }
    }

    /// ниже диначиеское представление
    // trait CommonRecog2 {
    //     type Output;
    //     fn recog(code: &Code) -> Result<Self::Output, &'static str>
    //     where
    //         Self: Sized;
    // }

    // impl<T: EnumRecog> CommonRecog2 for Enum<T> {
    //     type Output = T::Output;
    //     fn recog(code: &Code) -> Result<Self::Output, &'static str>
    //     where
    //         Self: Sized,
    //     {
    //         T::cursor_aware_recog(code).map_err(|v| v[0])
    //     }
    // }

    // macro_rules! impl_enum_seq {
    //     ($($a:ident)+) => {
    //         impl<$($a: CommonRecog + DynSpanable),+> EnumRecog for ($($a),+) {
    //             impl_enum_seq!(@fn $($a)+);
    //         }
    //     };
    //     (@fn $a:ident $($other:ident)+) => {
    //         type Output = Box<dyn DynSpanable>;
    //         fn cursor_aware_recog(code: &Code) -> Result<Self::Output, Vec<&'static str>> {
    //             let mut errs = vec![];
    //             $a::recog(code)
    //                 .map_err(|e| errs.push(e))
    //                 .map(|v| {
    //                     Box::new(v) as Box<dyn DynSpanable>
    //                 })
    //             $(
    //                 .or_else(|_| {
    //                     $other::recog(code)
    //                         .map_err(|e| errs.push(e))
    //                         .map(|v| {
    //                             Box::new(v) as Box<dyn DynSpanable>
    //                         })
    //                 })
    //             )+
    //                 .map_err(|_| errs)

    //         }
    //     }
    // }

    // tuple_impl!(impl_enum_seq!);
    use super::{items::*, *};
    #[test]
    fn test() {
        assert_eq!(
            Item::cursor_aware_recog(&"tmp".into()).unwrap(),
            Item::Ident(Token::new(0..3))
        );
        assert_eq!(
            Item::cursor_aware_recog(&r#""n""#.into()).unwrap(),
            Item::String(Token::new(0..3))
        );

        assert_eq!(
            Enum::<Item>::recog(&mut r#""n""#.into()).unwrap(),
            Enum::new(Item::String(Token::new(0..3)))
        );
    }
}
use enum_::*;

mod args;
use args::*;

mod items;

mod construct;

mod tmp {
    use super::{items::*, *};

    #[test]
    fn test() {
        assert_eq!(
            Token::<Ident>::cursor_aware_recog(&"tmp".into()).unwrap(),
            Token::new(0..3)
        );
        assert_eq!(
            Token::<String>::cursor_aware_recog(&r#""n""#.into()).unwrap(),
            Token::new(0..3)
        );
        assert_eq!(
            IdentString::cursor_aware_recog(&r#"tmp"n"tmp"#.into()).unwrap(),
            IdentString(Token::new(0..3), Token::new(3..6), Token::new(6..9))
        );
        assert_eq!(
            <(IdentString, Token<String>) as SequenceRecog>::cursor_aware_recog(
                &r#"tmp"n"tmp"n""#.into()
            )
            .unwrap(),
            (
                IdentString(Token::new(0..3), Token::new(3..6), Token::new(6..9)),
                Token::new(9..12),
            )
        );
        assert_eq!(
            <((Token<Ident>, Token<String>), Token<Ident>) as SequenceRecog>::cursor_aware_recog(
                &r#"tmp"n"tmp"#.into()
            )
            .unwrap(),
            ((Token::new(0..3), Token::new(3..6)), Token::new(6..9))
        );
    }
}
