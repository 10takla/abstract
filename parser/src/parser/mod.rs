use super::utils::args::*;
use colored::Colorize;
use paste::paste;
use regex::Regex;
use std::{any::Any, cell::RefCell, fmt::Arguments, marker::PhantomData, ops::Range};

pub type Slice = Range<usize>;

pub trait Spanable {
    fn span(&self) -> Slice;

    fn span_by_cursor(&self, cursor: usize) -> Slice {
        let v = self.span();
        cursor + v.start..cursor + v.end
    }
}

mod cache {
    use super::{wrapper::CommonRecog, *};
    use std::{
        any::{Any, TypeId},
        collections::{HashMap, HashSet},
        ops::{Deref, DerefMut},
        rc::Rc,
    };

    #[derive(Debug, Clone)]
    pub struct Cachable<T>(PhantomData<T>);

    impl<T: CommonRecog<Output: Clone> + 'static> CommonRecog for Cachable<T> {
        type Output = T::Output;
        fn recog(ctxt: &Ctxt) -> Result<Self::Output, CommonError> {
            // `{...}` becouse to shorten the lifetime of cache borrowing
            {
                if let Some(v) = Self::check_cache(ctxt.cache.borrow().deref(), ctxt.code.cursor) {
                    return v.clone();
                }
            }
            let v = T::recog(ctxt);
            Self::set_cache(
                ctxt.cache.borrow_mut().deref_mut(),
                ctxt.code.cursor,
                v.clone(),
            );
            v
        }
    }

    impl<T: CommonRecog + 'static> Cachable<T> {
        fn check_cache<'a>(
            cache: &'a Cache,
            cursor: usize,
        ) -> Option<&'a Result<T::Output, CommonError>> {
            cache
                .get(&(cursor, TypeId::of::<T>()))
                .and_then(|v| v.downcast_ref())
        }

        fn set_cache(cache: &mut Cache, cursor: usize, v: Result<T::Output, CommonError>) {
            cache.insert((cursor, TypeId::of::<T>()), Box::new(v));
        }
    }

    pub trait CacheRecog: CommonTrait {
        fn cache_recog(ctxt: &Ctxt) -> Result<Self::Output, CommonError>
        where
            Self: CommonRecog<Output: Clone> + 'static + Sized,
        {
            <Cachable<Self>>::recog(ctxt)
        }
    }

    impl<T: CommonRecog> CacheRecog for T {}
}
pub use cache::*;

mod wrapper {
    use super::*;
    use crate::tuple_impl;
    use macros::Spanable;
    use std::{
        fmt::{Debug, Display},
        rc::Rc,
    };

    #[derive(Debug, Clone, PartialEq)]
    pub enum CommonError {
        Token(TokenError),
        Enum(Vec<CommonError>),
        Seq(Box<CommonError>),
        NotPredicate(Slice),
        OneOrMore(usize, Vec<Result<String, CommonError>>),
    }

    impl CommonError {
        pub fn start(&self) -> Result<usize, ()> {
            match self {
                CommonError::Enum(v) => v.iter().map(Self::start).max().unwrap(),
                CommonError::Seq(v) => v.start(),
                CommonError::Token(v) => match v {
                    TokenError::CommonTokenError(s, _) => Ok(s.start),
                    TokenError::LineOver => Err(()),
                },
                CommonError::NotPredicate(v) => Ok(v.start),
                CommonError::OneOrMore(v, ..) => Ok(*v),
            }
        }
        pub fn end(&self) -> Result<usize, ()> {
            match self {
                CommonError::Enum(v) => v.iter().map(Self::end).max().unwrap(),
                CommonError::Seq(v) => v.end(),
                CommonError::Token(v) => match v {
                    TokenError::CommonTokenError(s, _) => Ok(s.end),
                    TokenError::LineOver => Err(()),
                },
                CommonError::NotPredicate(v) => Ok(v.end),
                CommonError::OneOrMore(v, ..) => Ok(*v + 1),
            }
        }

        pub fn span(&self, source: &str) -> Slice {
            match self {
                CommonError::Enum(v) => v
                    .iter()
                    .map(|v| v.span(source))
                    .max_by(|a, b| a.end.cmp(&b.end))
                    .unwrap(),
                CommonError::Seq(v) => v.span(source),
                CommonError::Token(v) => match v {
                    TokenError::CommonTokenError(s, _) => s.clone(),
                    TokenError::LineOver => source.len()..source.len(),
                },
                CommonError::NotPredicate(v) => v.clone(),
                CommonError::OneOrMore(v, ..) => *v..*v + 1,
            }
        }
    }

    pub trait CommonTrait: std::fmt::Debug + Clone {}
    impl<T: std::fmt::Debug + Clone> CommonTrait for T {}

    pub trait CommonRecog: CommonTrait {
        type Output: CommonTrait;
        fn recog(ctxt: &Ctxt) -> Result<Self::Output, CommonError>;
    }

    impl<T: EnumRecog> CommonRecog for T {
        type Output = T::Output;
        fn recog(ctxt: &Ctxt) -> Result<Self::Output, CommonError> {
            T::cursor_aware_recog(ctxt).map_err(CommonError::Enum)
        }
    }

    #[derive(Debug, PartialEq, Clone)]
    pub struct Seq<T>(pub T);

    impl<T: SequenceRecog> CommonRecog for Seq<T> {
        type Output = T::Output;
        fn recog(ctxt: &Ctxt) -> Result<Self::Output, CommonError> {
            T::cursor_aware_recog(ctxt).map_err(|v| CommonError::Seq(Box::new(v)))
        }
    }
    macro_rules! seq_impl {
        ($($a:ident)+) => {
            impl<$($a),+> CommonRecog for ($($a),+)
            where
                ($($a),+): SequenceRecog
            {
                type Output = <($($a),+) as SequenceRecog>::Output;
                fn recog(ctxt: &Ctxt) -> Result<Self::Output, CommonError> {
                    <($($a),+)>::cursor_aware_recog(ctxt)
                }
            }
        };
    }
    tuple_impl!(seq_impl!);

    impl<T> CommonRecog for Token<T>
    where
        Self: TokenRecog<Inner = T>,
    {
        type Output = Self;
        fn recog(ctxt: &Ctxt) -> Result<Self::Output, CommonError> {
            Self::cursor_aware_recog(&ctxt.code).map_err(CommonError::Token)
        }
    }

    impl<T: CommonTrait, I: IterRepetition> CommonRecog for ZeroOrMore<T, I>
    where
        Self: RepetitionRecog,
    {
        type Output = RepOk<<<ZeroOrMore<T, I> as RepetitionRecog>::Item as CommonRecog>::Output>;
        fn recog(ctxt: &Ctxt) -> Result<Self::Output, CommonError> {
            let (a, b) = Self::cursor_aware_recog(ctxt);
            a.map(|a| (a, b))
        }
    }
    impl<T: CommonTrait, I: IterRepetition> CommonRecog for OneOrMore<T, I>
    where
        OneOrMore<T, I>: RepetitionRecog,
    {
        type Output = RepOk<<<OneOrMore<T, I> as RepetitionRecog>::Item as CommonRecog>::Output>;
        fn recog(ctxt: &Ctxt) -> Result<Self::Output, CommonError> {
            let (a, b) = Self::cursor_aware_recog(ctxt);
            a.map(|a| (a, b))
        }
    }

    #[derive(Debug, Clone)]
    pub struct AndPredicate<T: CommonRecog>(T::Output, PhantomData<T>);

    impl<T: CommonRecog<Output: Spanable>> CommonRecog for AndPredicate<T> {
        type Output = AndPredicate<T>;
        fn recog(ctxt: &Ctxt) -> Result<Self::Output, CommonError> {
            T::recog(ctxt).map(|v| AndPredicate(v, PhantomData))
        }
    }

    impl<T: CommonRecog + Spanable> Spanable for AndPredicate<T> {
        fn span(&self) -> Slice {
            0..0
        }
    }

    #[derive(Debug, Clone)]
    pub struct NotPredicate<T>(PhantomData<T>);

    impl<T: CommonRecog<Output: Spanable>> CommonRecog for NotPredicate<T> {
        type Output = ();
        fn recog(ctxt: &Ctxt) -> Result<Self::Output, CommonError> {
            if let Ok(v) = T::recog(ctxt) {
                Err(CommonError::NotPredicate(v.span()))
            } else {
                Ok(())
            }
        }
    }

    impl Spanable for () {
        fn span(&self) -> Slice {
            0..0
        }
    }

    impl<T: CommonRecog> CommonRecog for Option<T> {
        type Output = Option<T::Output>;
        fn recog(ctxt: &Ctxt) -> Result<Self::Output, CommonError> {
            Ok(T::recog(ctxt).ok())
        }
    }

    impl<T: Spanable> Spanable for Option<T> {
        fn span(&self) -> Slice {
            self.as_ref().map(Spanable::span).unwrap_or_default()
        }
    }
}
pub use wrapper::*;

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

    pub trait RegularToken: CommonTrait {
        const REG_EXPR: &'static str;
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum TokenError {
        CommonTokenError(Slice, CommonTokenError),
        LineOver,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum CommonTokenError {
        CurrentErrors(&'static str),
        RegularToken(&'static str),
    }

    pub trait TokenRecog: CommonTrait {
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
        fn span_by_cursor(&self, cursor: usize) -> Slice {
            self.span()
        }
    }
}
pub use token::*;

mod sequence {
    use super::*;
    use macros::spanable;
    use std::process::Output;

    pub trait SequenceRecog: CommonTrait {
        type Output: CommonTrait;
        // распознавнаие с продвижением курсора
        fn cursor_aware_recog(ctxt: &Ctxt) -> Result<Self::Output, CommonError> {
            Self::structure_assembling(&mut ctxt.clone())
        }

        fn structure_assembling(ctxt: &mut Ctxt) -> Result<Self::Output, CommonError>;

        fn promotion<T: CommonRecog>(ctxt: &mut Ctxt) -> Result<T::Output, CommonError>
        where
            T::Output: Spanable,
        {
            T::recog(ctxt).map(|v| {
                ctxt.code.cursor += v.span().len();
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
            // impl_seq!(@spanable $($a)+);
            impl<$($a: CommonRecog),+> SequenceRecog for ($($a),+)
            where
                ($($a),+): CommonTrait,
                ($($a::Output),+): CommonTrait,
                $($a::Output: Spanable),+
            {
                type Output = ($($a::Output),+);
                fn structure_assembling(ctxt: &mut Ctxt) -> Result<Self::Output, CommonError> {
                    Ok(($(Self::promotion::<$a>(ctxt)?),+))
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
    spanable!(T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15 T16 T17);

    use items::*;
    #[test]
    fn test() {
        assert_eq!(
            <Seq<(Token<Ident>, Token<String>)>>::recog(&r#"tmp"n"tmp"n""#.into()).unwrap(),
            (Token::new(0..3), Token::new(3..6),)
        );
        assert_eq!(
            <Seq<(Seq<(Token<Ident>, Token<String>)>, Token<Ident>)>>::recog(
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
pub use sequence::*;

// mod dyn_sequence;
// use dyn_sequence::*;

mod enum_ {
    use super::*;
    use crate::tuple_impl;

    pub trait EnumRecog: CommonTrait {
        type Output: CommonTrait;
        fn cursor_aware_recog(ctxt: &Ctxt) -> Result<Self::Output, Vec<CommonError>> {
            let mut errs = vec![];
            Self::structure_assembling(ctxt)
                .into_iter()
                .find_map(|v| v().map_err(|e| errs.push(e)).ok())
                .ok_or(errs)
        }
        fn structure_assembling<'a>(
            ctxt: &'a Ctxt,
        ) -> Vec<Box<dyn core::ops::Fn() -> Result<Self::Output, CommonError> + 'a>>;
    }

    /// ниже диначиеское представление
    // trait CommonRecog2 {
    //     type Output;
    //     fn recog(ctxt: &Ctxt) -> Result<Self::Output, &'static str>
    //     where
    //         Self: Sized;
    // }

    // impl<T: EnumRecog> CommonRecog2 for Enum<T> {
    //     type Output = T::Output;
    //     fn recog(ctxt: &Ctxt) -> Result<Self::Output, &'static str>
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
    //         fn cursor_aware_recog(ctxt: &Ctxt) -> Result<Self::Output, Vec<&'static str>> {
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
pub use enum_::*;

mod repetiotion {
    use super::*;
    use items::*;
    use std::{default, fmt::Debug, iter, ops::ControlFlow};

    impl CommonRecog for () {
        type Output = ();
        fn recog(ctxt: &Ctxt) -> Result<Self::Output, CommonError> {
            Err(CommonError::Token(TokenError::LineOver))
        }
    }

    pub type RepOk<T> = (Vec<T>, Vec<Result<T, CommonError>>);
    pub type RepOutput<T> = (Result<Vec<T>, CommonError>, (Vec<Result<T, CommonError>>));

    pub trait RepetitionRecog {
        type Item: CommonRecog<Output: Spanable>;
        fn cursor_aware_recog(ctxt: &Ctxt) -> RepOutput<<Self::Item as CommonRecog>::Output>;
    }
    pub trait IterRepetition: CommonTrait {
        fn iter(
            ctxt: &Ctxt,
            result: impl FnMut() -> Result<(), ControlFlow<()>>,
        ) -> ControlFlow<()>;
    }

    #[derive(Debug, Clone)]
    pub struct OneOrMore<T, I = ()>(PhantomData<T>, PhantomData<I>);

    impl<T: CommonRecog<Output: Spanable>, I: IterRepetition> RepetitionRecog for OneOrMore<T, I>
    where
        ZeroOrMore<T, I>: RepetitionRecog<Item = T>,
        
    {
        type Item = T;
        fn cursor_aware_recog(ctxt: &Ctxt) -> RepOutput<<Self::Item as CommonRecog>::Output> {
            let (vec, coll) = <ZeroOrMore<T, I>>::cursor_aware_recog(ctxt);
            let vec = vec.unwrap();
            if vec.is_empty() {
                (Err(CommonError::OneOrMore(ctxt.code.cursor, coll.clone().into_iter().map(|v| v.map(|v| format!("{v:?}"))).collect())), coll)
            } else {
                (Ok(vec), coll)
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct ZeroOrMore<T, I = ()>(PhantomData<T>, PhantomData<I>);

    impl<T: CommonRecog<Output: Spanable>, I: IterRepetition> RepetitionRecog for ZeroOrMore<T, I> {
        type Item = T;
        fn cursor_aware_recog(ctxt: &Ctxt) -> RepOutput<<Self::Item as CommonRecog>::Output> {
            let (mut pass, mut fail_collection) = (vec![], vec![]);
            let mut ctxt2 = (*ctxt).clone();
            (0..).try_for_each(|_| {
                if ctxt2.code.cursor >= ctxt2.code.source.len() {
                    return ControlFlow::Break(());
                }
                I::iter(&ctxt2.clone(), || {
                    Self::Item::recog(&ctxt2)
                        .map_err(|e| {
                            if let Ok(v) = e.end() {
                                ctxt2.code.cursor = v;
                                fail_collection.push(Err(e));
                                ControlFlow::Continue(())
                            } else {
                                ControlFlow::Break(())
                            }
                        })
                        .and_then(|item| {
                            if item.span().len() == 0 {
                                return Err(ControlFlow::Break(()));
                            }
                            ctxt2.code.cursor += item.span().len();
                            if fail_collection.is_empty() {
                                pass.push(item);
                            } else {
                                fail_collection.push(Ok(item));
                            }
                            Ok(())
                        })
                })
            });
            (Ok(pass), fail_collection)
        }
    }

    impl IterRepetition for () {
        fn iter(
            ctxt: &Ctxt,
            mut result: impl FnMut() -> Result<(), ControlFlow<()>>,
        ) -> ControlFlow<()> {
            if let Err(ControlFlow::Break(())) = result() {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct ErrorBreak;

    impl IterRepetition for ErrorBreak {
        fn iter(
            ctxt: &Ctxt,
            mut result: impl FnMut() -> Result<(), ControlFlow<()>>,
        ) -> ControlFlow<()> {
            if result().is_err() {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct BreakWhile<B> {
        break_: PhantomData<B>,
    }

    impl<B: CommonRecog> IterRepetition for BreakWhile<B> {
        fn iter(
            ctxt: &Ctxt,
            result: impl FnMut() -> Result<(), ControlFlow<()>>,
        ) -> ControlFlow<()> {
            if let Ok(v) = B::recog(&ctxt) {
                return ControlFlow::Break(());
            }
            <()>::iter(ctxt, result)
        }
    }

    impl<T: Spanable> Spanable for RepOk<T> {
        fn span(&self) -> Slice {
            self.0.span()
        }
        fn span_by_cursor(&self, cursor: usize) -> Slice {
            self.0.span_by_cursor(cursor)
        }
    }

    impl<T: Spanable> Spanable for Vec<T> {
        fn span(&self) -> Slice {
            let v = |v: &T| {
                let v = v.span();
                (!(v.start == 0 && v.end == 0)).then_some(v)
            };
            self.iter()
                .find_map(v)
                .zip(self.iter().rev().find_map(v))
                .map(|v| v.0.start..v.1.end)
                .unwrap_or_default()
        }
        fn span_by_cursor(&self, cursor: usize) -> Slice {
            self.first()
                .zip(self.last())
                .map(|v| v.0.span_by_cursor(cursor).start..v.1.span_by_cursor(cursor).end)
                .unwrap_or_default()
        }
    }

    // impl<T: Spanable + Clone + std::fmt::Debug> Spanable for (Vec<T>, Vec<CommonError>) {
    //     fn span(&self) -> Slice {
    //         if let Some(error) = self.1.first()
    //             && let Ok(error_start) = error.start()
    //         {
    //             self.0
    //                 .clone()
    //                 .into_iter()
    //                 .take_while(|item| item.span().start < error_start)
    //                 .collect::<Vec<_>>()
    //         } else {
    //             self.0.clone()
    //         }
    //         .span()
    //     }
    //     fn span_by_cursor(&self, cursor: usize) -> Slice {
    //         if let Some(error) = self.1.first()
    //             && let Ok(error_start) = error.start()
    //         {
    //             self.0
    //                 .clone()
    //                 .into_iter()
    //                 .take_while(|item| item.span().start < error_start)
    //                 .collect::<Vec<_>>()
    //         } else {
    //             self.0.clone()
    //         }
    //         .span_by_cursor(cursor)
    //     }
    // }

    #[test]
    fn test() {
        assert_eq!(
            <ZeroOrMore<Item>>::cursor_aware_recog(&r#""n""n""#.into()),
            (
                Ok(vec![
                    Item::String(Token::new(0..3)),
                    Item::String(Token::new(3..6))
                ]),
                vec![]
            )
        );
        assert_eq!(
            <ZeroOrMore<Token<String>>>::recog(&r#""n""n""#.into()).unwrap(),
            (vec![Token::new(0..3), Token::new(3..6)], vec![])
        );
    }
}
pub use repetiotion::*;

mod error {
    use super::*;
    use std::fmt::Display;
    impl CommonError {
        pub fn diag_display(&self, source: &str) -> String {
            match self {
                CommonError::Token(b) => b.tmp(source),
                CommonError::Enum(v) => v
                    .into_iter()
                    .map(|v| v.diag_display(source).to_string())
                    .collect::<Vec<_>>()
                    .join("\n"),
                CommonError::Seq(v) => v.diag_display(source),
                CommonError::NotPredicate(v) => tmp(v.clone(), source, "NotPredicate"),
                CommonError::OneOrMore(v, ..) => tmp(*v..*v + 1, source, "OneOrMore"),
            }
        }
    }

    impl TokenError {
        fn tmp(&self, source2: &str) -> String {
            let (slice2, m) = match self {
                TokenError::CommonTokenError(slice, m) => (slice.clone(), m),
                TokenError::LineOver => return format!("LineOver"),
            };
            tmp(slice2, source2, format!("{m:?}"))
        }
    }

    fn tmp(slice2: Slice, source2: &str, m: impl Display) -> String {
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
            format!("{}-Ожидается {m}", "^".repeat(slice2.len())).red()
        )
    }
}
pub use error::*;

mod items;

pub mod constructor;

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
    use macros::{EnumRecog, RegularToken, Spanable};

    #[derive(RegularToken, PartialEq, Debug, Clone)]
    #[reg_expr = r"\b[_a-zA-Z][_a-zA-Z0-9]*\b"]
    struct Ident;

    #[derive(RegularToken, PartialEq, Debug, Clone)]
    #[reg_expr = r#""[^"\\]*(?:\\.[^"\\]*)*""#]
    struct String;

    #[derive(RegularToken, PartialEq, Debug, Clone)]
    #[reg_expr = r"\b\d+\b"]
    struct Number;

    #[derive(EnumRecog, Spanable, PartialEq, Debug, Clone)]
    enum Literal {
        Number(Token<Number>),
        String(Token<String>),
    }

    #[derive(EnumRecog, Spanable, PartialEq, Debug, Clone)]
    enum Item {
        #[ty((Literal, Token<Ident>, ZeroOrMore<Literal>))]
        LiteralIdent((Literal, Token<Ident>, RepOk<Literal>)),
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
                (
                    vec![
                        Literal::String(Token::new(9..15)),
                        Literal::String(Token::new(15..21))
                    ],
                    vec![]
                )
            ))
        );
    }
}
