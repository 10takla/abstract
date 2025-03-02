use colored::Colorize;
use paste::paste;
use regex::Regex;
use std::{any::Any, cell::RefCell, fmt::Arguments, marker::PhantomData, ops::Range};

type Slice = Range<usize>;

pub trait Spanable {
    fn span(&self) -> Slice;
}

struct Ctxt<'a> {
    code: &'a Code<'a>,
    logger: (Print, RefCell<usize>),
}

mod cache {
    use super::{wrapper::CommonRecog, *};
    use std::{
        any::{Any, TypeId},
        collections::{HashMap, HashSet},
    };

    pub trait CacheRecog: CommonRecog + Sized + 'static {
        fn cache_recog(
            ctxt: Ctxt,
            cache: &mut HashMap<(usize, TypeId), Box<dyn Any>>,
        ) -> Result<<Self as CommonRecog>::Output, CommonError>
        where
            <Self as CommonRecog>::Output: Clone,
        {
            if let Some(v) = Self::check_cache(ctxt.code, cache)
                .and_then(|v| v.downcast_ref::<<Self as CommonRecog>::Output>())
            {
                Ok((*v).clone())
            } else {
                Self::recog(ctxt.code).map(|v| {
                    Self::set_cache(v.clone(), ctxt.code, cache);
                    v
                })
            }
        }

        fn check_cache<'a>(
            code: &Code,
            cache: &'a HashMap<(usize, TypeId), Box<dyn Any>>,
        ) -> Option<&'a Box<dyn Any>> {
            cache.get(&(code.cursor, TypeId::of::<Self>()))
        }

        fn set_cache(
            v: <Self as CommonRecog>::Output,
            code: &Code,
            cache: &mut HashMap<(usize, TypeId), Box<dyn Any>>,
        ) {
            cache.insert((code.cursor, TypeId::of::<Self>()), Box::new(v));
        }
    }

    impl<T: CommonRecog + Clone + 'static> CacheRecog for T {}
}
use cache::*;

mod wrapper {
    use super::*;
    use crate::tuple_impl;
    use std::fmt::Display;

    #[derive(Debug, Clone)]
    pub enum CommonError {
        Token(TokenError),
        Enum(Vec<CommonError>),
        Seq(Box<CommonError>),
    }

    pub trait CommonRecog {
        type Output;
        fn recog(code: &Code) -> Result<Self::Output, CommonError>;
    }

    impl<T: EnumRecog> CommonRecog for T {
        type Output = T::Output;
        fn recog(code: &Code) -> Result<Self::Output, CommonError> {
            T::cursor_aware_recog(code).map_err(CommonError::Enum)
        }
    }

    #[derive(Debug, PartialEq)]
    pub struct Seq<T>(pub T);

    impl<T: SequenceRecog> CommonRecog for Seq<T> {
        type Output = T::Output;
        fn recog(code: &Code) -> Result<Self::Output, CommonError> {
            T::cursor_aware_recog(code).map_err(|v| CommonError::Seq(Box::new(v)))
        }
    }
    macro_rules! seq_impl {
        ($($a:ident)+) => {
            impl<$($a),+> CommonRecog for ($($a),+)
            where
                ($($a),+): SequenceRecog
            {
                type Output = <($($a),+) as SequenceRecog>::Output;
                fn recog(code: &Code) -> Result<Self::Output, CommonError> {
                    <($($a),+)>::cursor_aware_recog(code)
                }
            }
        };
    }
    tuple_impl!(seq_impl!);

    impl<T> CommonRecog for Token<T>
    where
        Token<T>: TokenRecog<Inner = T>,
    {
        type Output = Self;
        fn recog(code: &Code) -> Result<Self::Output, CommonError> {
            Self::cursor_aware_recog(code).map_err(CommonError::Token)
        }
    }

    impl<T> CommonRecog for Vec<T>
    where
        Vec<T>: RepetitionRecog,
    {
        type Output = Vec<<<Vec<T> as RepetitionRecog>::Item as CommonRecog>::Output>;
        fn recog(code: &Code) -> Result<Self::Output, CommonError> {
            Ok(Self::cursor_aware_recog(code))
        }
    }
    impl<T, B> CommonRecog for BreakRepetition<T, B>
    where
        BreakRepetition<T, B>: RepetitionRecog,
    {
        type Output =
            Vec<<<BreakRepetition<T, B> as RepetitionRecog>::Item as CommonRecog>::Output>;
        fn recog(code: &Code) -> Result<Self::Output, CommonError> {
            Ok(Self::cursor_aware_recog(code))
        }
    }
}
use wrapper::*;

mod token {
    use super::*;
    use core::prelude::v1;

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

    #[derive(Debug, Clone)]
    pub enum TokenError {
        CommonTokenError(Slice, CommonTokenError),
        LineOver,
    }

    #[derive(Debug, Clone)]
    pub enum CommonTokenError {
        CurrentErrors(&'static str),
        RegularToken(&'static str),
    }

    pub trait TokenRecog {
        type Inner;
        // распознавание относительно курсора, то есть с учетом смещения строки, без продвижения
        fn cursor_aware_recog(code: &Code) -> Result<Token<Self::Inner>, TokenError> {
            Self::start_string_aware_recog(&code.source[code.cursor..])
                .map(|span| Token {
                    _marker: PhantomData,
                    span: code.cursor + span.start..code.cursor + span.end,
                })
                .map_err(|v1| match v1 {
                    TokenError::CommonTokenError(span, v) => TokenError::CommonTokenError(
                        code.cursor + span.start..code.cursor + span.end,
                        v,
                    ),
                    _ => v1,
                })
        }

        // расознавания происходит относительно строки, без каких либо смещений по курсору
        fn start_string_aware_recog(code: &str) -> Result<Slice, TokenError>;
    }

    impl<T: RegularToken> TokenRecog for Token<T> {
        type Inner = T;
        fn start_string_aware_recog(code: &str) -> Result<Slice, TokenError> {
            if code.is_empty() {
                return Err(TokenError::LineOver);
            }

            Regex::new(&format!("^{}", T::REG_EXPR))
                .unwrap()
                .find(code)
                .map(|mat| mat.range())
                .ok_or(TokenError::CommonTokenError(
                    {
                        let (i, ch) = code.char_indices().next().unwrap();
                        i..i + ch.len_utf8()
                    },
                    CommonTokenError::RegularToken(T::REG_EXPR),
                ))
        }
    }

    impl<T> Spanable for Token<T> {
        fn span(&self) -> Slice {
            self.span.clone()
        }
    }
}
use token::*;

mod sequence {
    use super::*;
    use std::process::Output;

    pub trait SequenceRecog {
        type Output;
        // распознавнаие с продвижением курсора
        fn cursor_aware_recog(code: &Code) -> Result<Self::Output, CommonError> {
            Self::structure_assembling(&mut code.clone())
        }

        fn structure_assembling(code: &mut Code) -> Result<Self::Output, CommonError>;

        fn promotion<T: CommonRecog>(code: &mut Code) -> Result<T::Output, CommonError>
        where
            T::Output: Spanable,
        {
            T::recog(code).map(|v| {
                code.cursor += v.span().len();
                v
            })
        }
    }

    #[macro_export]
    macro_rules! tuple_impl {
        ($macros:ident! $(@$key:ident)?) => {
            tuple_impl!(@recursive $macros! $(@$key)? T T T T T T T T T T T T T T T T T T);
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
            impl<$($a: CommonRecog),+> SequenceRecog for ($($a),+)
            where
                $($a::Output: Spanable),+
            {
                type Output = ($($a::Output),+);
                fn structure_assembling(code: &mut Code) -> Result<Self::Output, CommonError> {
                    Ok(($(Self::promotion::<$a>(code)?),+))
                }
            }
        };
        (@spanable $a:ident $($other:ident)+) => {
            impl<$a: Spanable, $($other: Spanable),+> Spanable for ($a, $($other),+) {
                fn span(&self) -> Slice {
                    self.0.span().start..self.${count($other)}.span().end
                }
            }
        };
    }
    tuple_impl!(impl_seq! @impl);

    use items::*;
    #[test]
    fn test() {
        assert_eq!(
            Seq::<(Token<Ident>, Token<String>)>::recog(&r#"tmp"n"tmp"n""#.into()).unwrap(),
            (Token::new(0..3), Token::new(3..6),)
        );
        assert_eq!(
            Seq::<(Seq<(Token<Ident>, Token<String>)>, Token<Ident>)>::recog(
                &r#"tmp"n"tmp"n""#.into()
            )
            .unwrap(),
            ((Token::new(0..3), Token::new(3..6)), Token::new(6..9))
        );

        assert_eq!(
            <(Seq<IdentString>, Token<String>)>::cursor_aware_recog(&r#"tmp"n"tmp"n""#.into())
                .unwrap(),
            (
                IdentString(Token::new(0..3), Token::new(3..6), Token::new(6..9)),
                Token::new(9..12),
            )
        );
        assert_eq!(
            <(Seq<(Token<Ident>, Token<String>)>, Token<Ident>)>::cursor_aware_recog(
                &r#"tmp"n"tmp"#.into()
            )
            .unwrap(),
            ((Token::new(0..3), Token::new(3..6)), Token::new(6..9))
        );
        assert_eq!(
            <((Token<Ident>, Token<String>), Token<Ident>)>::cursor_aware_recog(
                &r#"tmp"n"tmp"#.into()
            )
            .unwrap(),
            ((Token::new(0..3), Token::new(3..6)), Token::new(6..9))
        );
    }
}
use sequence::*;

// mod dyn_sequence;
// use dyn_sequence::*;

mod enum_ {
    use super::*;
    use crate::tuple_impl;

    pub trait EnumRecog {
        type Output;
        fn cursor_aware_recog(code: &Code) -> Result<Self::Output, Vec<CommonError>> {
            let mut errs = vec![];
            Self::structure_assembling(code)
                .into_iter()
                .find_map(|v| v().map_err(|e| errs.push(e)).ok())
                .ok_or(errs)
        }
        fn structure_assembling<'a>(
            code: &'a Code,
        ) -> Vec<Box<dyn core::ops::Fn() -> Result<Self::Output, CommonError> + 'a>>;
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
            Item::recog(&r#""n""#.into()).unwrap(),
            Item::String(Token::new(0..3))
        );
    }
}
use enum_::*;

mod repetiotion {
    use super::*;
    use items::*;
    use std::{fmt::Debug, ops::ControlFlow};

    type Items = Vec<Item>;

    impl CommonRecog for () {
        type Output = ();
        fn recog(code: &Code) -> Result<Self::Output, CommonError> {
            Err(CommonError::Token(TokenError::LineOver))
        }
    }

    pub trait RepetitionRecog {
        type Item: CommonRecog<Output: Spanable>;

        fn cursor_aware_recog(code: &Code) -> Vec<<Self::Item as CommonRecog>::Output> {
            let mut vec = vec![];
            let mut code = code.clone();
            loop {
                let Ok(item) = Self::Item::recog(&code) else {
                    break;
                };
                code.cursor = item.span().end;
                vec.push(item);
            }
            vec
        }
    }

    pub struct BreakRepetition<T, B> {
        inner: Vec<T>,
        break_: PhantomData<B>,
    }

    impl<T: CommonRecog<Output: Spanable>, B: CommonRecog> RepetitionRecog for BreakRepetition<T, B> {
        type Item = T;
        fn cursor_aware_recog(code: &Code) -> Vec<<Self::Item as CommonRecog>::Output> {
            let mut vec = vec![];
            let mut code = code.clone();
            loop {
                if let Ok(v) = B::recog(&code) {
                    break;
                }

                if let Ok(item) = Self::Item::recog(&code) {
                    code.cursor = item.span().end;
                    vec.push(item);
                } else {
                    break;
                }
            }
            vec
        }
    }

    impl<T: Spanable, B> Spanable for BreakRepetition<T, B> {
        fn span(&self) -> Slice {
            self.inner.span()
        }
    }

    impl<T: CommonRecog<Output: Spanable>> RepetitionRecog for Vec<T> {
        type Item = T;
    }

    impl<T: Spanable> Spanable for Vec<T> {
        fn span(&self) -> Slice {
            self.first()
                .zip(self.last())
                .map(|v| v.0.span().start..v.1.span().end)
                .unwrap_or_default()
        }
    }

    #[test]
    fn test() {
        assert_eq!(
            Vec::<Item>::cursor_aware_recog(&r#""n""n""#.into()),
            vec![
                Item::String(Token::new(0..3)),
                Item::String(Token::new(3..6))
            ]
        );
        assert_eq!(
            <Vec::<Token<String>>>::recog(&r#""n""n""#.into()).unwrap(),
            vec![Token::new(0..3), Token::new(3..6)]
        );
    }
}
use repetiotion::*;

mod error {
    use super::*;
    impl CommonError {
        pub fn diag_display(&self, slice: Slice, source: &str) -> String {
            match self {
                CommonError::Token(b) => b.tmp(source),
                CommonError::Enum(v) => v
                    .into_iter()
                    .map(|v| v.diag_display(slice.clone(), source).to_string())
                    .collect::<Vec<_>>()
                    .join("\n"),
                CommonError::Seq(v) => v.diag_display(slice.clone(), source),
            }
        }
    }

    impl TokenError {
        pub fn tmp(&self, source2: &str) -> String {
            let (slice2, m) = match self {
                TokenError::CommonTokenError(slice, m) => (slice.clone(), m),
                TokenError::LineOver => return format!("LineOver"),
            };

            let slice = {
                let v = source2[..slice2.start].chars().count();
                v..v + source2[slice2.clone()].chars().count() + 1
            };

            let mut iter = source2.split_inclusive('\n').enumerate();
            let mut get_line = |pos| {
                let mut acc = 0;
                iter.find_map(|(i, str)| {
                    acc += str.chars().count();
                    (pos < acc).then_some(i + 1)
                })
                .unwrap_or_default()
            };

            let f = |v: &[char]| v.iter().collect::<std::string::String>();

            let source = source2.chars().collect::<Vec<_>>();

            let (l, b, [min, max]) = ("|".blue(), "...", [0, 4]);

            let [distr_after, distr_before] = [
                slice.start > min,
                source.len().saturating_sub(slice.end - 1) != 0
                    && source.len() - 1 - slice.end - 1 >= max,
            ]
            .map(|cond| cond.then_some(b).unwrap_or_default());

            let code = format!(
                "{}{}{}",
                f(&source
                    .get({
                        let i = slice.start;
                        let r = if i < min { 0 } else { i - min };
                        r..i
                    })
                    .unwrap_or_default()),
                f(&source
                    .get(slice.clone())
                    .unwrap_or(&[*source.last().unwrap()]))
                .underline()
                .red(),
                f(&source
                    .get({
                        let i = slice.end - 1;
                        if source.len().saturating_sub(i) == 0 {
                            i..source.len()
                        } else {
                            i + 1..if source.len() - 1 - i < max {
                                source.len()
                            } else {
                                i + max
                            }
                        }
                    })
                    .unwrap_or_default())
            );

            let front_p = 3;
            let f = " ".repeat(front_p);

            format!(
                "
    {f}{l}
    {}{l} {}{code}{}
    {f}{l} {}{}
    ",
                format!("{:width$} ", get_line(slice.end - 1), width = front_p - 1),
                distr_after.blue(),
                distr_before.blue(),
                " ".repeat(min + distr_after.chars().count()),
                format!("{}-Ожидается {self:?}", "^".repeat(slice2.len())).red()
            )
        }
    }
}
use error::*;

mod args;
use crate::lexer2::print::Print;
use args::*;

mod items;

mod constructor;

mod tmp {
    use super::{items::*, *};

    #[test]
    fn test() {
        assert_eq!(
            <Token<Ident>>::cursor_aware_recog(&"tmp".into()).unwrap(),
            Token::new(0..3)
        );
        assert_eq!(
            <Token<String>>::cursor_aware_recog(&r#""n""#.into()).unwrap(),
            Token::new(0..3)
        );
        assert_eq!(
            IdentString::cursor_aware_recog(&r#"tmp"n"tmp"#.into()).unwrap(),
            IdentString(Token::new(0..3), Token::new(3..6), Token::new(6..9))
        );
    }
}

mod model {
    use super::*;
    use lexer3_macros::{EnumRecog, RegularToken, Spanable};

    #[derive(RegularToken, PartialEq, Debug)]
    #[reg_expr = r"\b[_a-zA-Z][_a-zA-Z0-9]*\b"]
    struct Ident;

    #[derive(RegularToken, PartialEq, Debug)]
    #[reg_expr = r#""[^"\\]*(?:\\.[^"\\]*)*""#]
    struct String;

    #[derive(RegularToken, PartialEq, Debug)]
    #[reg_expr = r"\b\d+\b"]
    struct Number;

    #[derive(EnumRecog, Spanable, PartialEq, Debug)]
    enum Literal {
        Number(Token<Number>),
        String(Token<String>),
    }

    #[derive(EnumRecog, Spanable, PartialEq, Debug)]
    enum Item {
        LiteralIdent((Literal, Token<Ident>, Vec<Literal>)),
        Literal(Literal),
        Ident(Token<Ident>),
    }

    #[test]
    fn test() {
        assert_eq!(
            Item::recog(&r#"code"#.into()).unwrap(),
            Item::Ident(Token::new(0..4))
        );
        assert_eq!(
            Item::recog(&r#""code""#.into()).unwrap(),
            Item::Literal(Literal::String(Token::new(0..6)))
        );
        assert_eq!(
            Item::recog(&r#""code"sdf"code""code""#.into()).unwrap(),
            Item::LiteralIdent((
                Literal::String(Token::new(0..6)),
                Token::new(6..9),
                vec![
                    Literal::String(Token::new(9..15)),
                    Literal::String(Token::new(15..21))
                ]
            ))
        );
    }
}
