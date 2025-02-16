//! ## Виды структур для парсинга:
//!
//! Существует 4 типа структуры:
//! - Основные 3 на основе одной позиции `cursor`:
//!     - `Token`
//!     - `Enum`
//!     - `Construct`
//! - На основе последовательности элементов основных типов:
//!     - `Items`
//!
//! ## Кеширование
//! ### Проверка из кеша происходит:
//! Из 4 существующих типов структур кеширование по единной позици `cursor` работает для:
//!     - `Token` - для `Construct(`Token`, ❌), Token` кеш `Token` будет исопльзоватся
//!     - `Enum` - для `Construct(`Enum`, ❌), Enum` кеш `Enum` будет исопльзоватся
//!     - `Construct` - необходимо кешировать как сам `Construct` так и последовательность его элементов
//! Примечание: на основе последовательности элементов кеширование невозможно, поэтому парсинг каждого элемента из позиции не будет смотреть на кеш предыдущей позиции. Поэтому тип `Items` будет очищать кеш с каждой позции (Это не виляет на алгоритм, только оптимизирует память и поиск кеша).
//!
//! ### Сохранение в кеш происходит:
//! Нет смысла запомниать элементы на уровне `Token` или `Enum`, ведь при парсинге, смотреть в кеш имеет смысл только если парсится `Construct`, так как любая другая структура предполагает единственный исход. Из этого слудет, что элементы заносятся в кеш на уровне элементов `Construct`.
//!
//! Единая структура для всех типов кеширования, основана на максимально сложном кеше - на кеше `Construct`. То есть это список, который содержит последовательностей элементов
//!

pub mod cache_and_diags;
pub mod code;

mod constructor;
pub use constructor::*;

mod print;
mod tests;

use cache_and_diags::{diag::Diag, Cache, CacheAndDiags, PassList};
use code::{Code, Source};
use colored::Colorize;
use constructor::*;
use macros::{constructor, parse_test};
use paste::paste;
use print::{colored, tmp_pass_or_fail, Print};
use regex::Regex;
use regex_automata::{
    dfa::{dense::DFA, Automaton},
    nfa::thompson::NFA,
    util::start::Config,
    Input,
};
use regex_syntax::{
    ast::{parse::Parser as AstParser, print::Printer},
    Parser,
};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    hash::DefaultHasher,
    io::{self, stdout, Cursor, Read, Write},
    ops::{ControlFlow, RangeInclusive},
    option::Iter,
    rc::Rc,
    sync::{Arc, Mutex},
    vec::IntoIter,
};
use std_reset::prelude::Deref;

#[derive(Clone, Debug)]
pub struct ParseArgs {
    code: Code,
    pub c_a_d: Arc<RefCell<CacheAndDiags>>,
    print: Print,
}

impl From<&str> for ParseArgs {
    fn from(value: &str) -> Self {
        Self::new(&value)
    }
}

impl From<(&str, Print)> for ParseArgs {
    fn from((source, print): (&str, Print)) -> Self {
        Self {
            code: source.into(),
            c_a_d: Default::default(),
            print,
        }
    }
}

impl ParseArgs {
    fn new(source: &str) -> Self {
        Self {
            code: source.into(),
            c_a_d: Default::default(),
            print: Default::default(),
        }
    }

    fn get_head(&self, name: &str, const_: impl Debug, pos: usize) -> std::string::String {
        format!(
            "{name} {const_:?}({pos}){}",
            if self.print.cache {
                format!(" {:?}", self.c_a_d.borrow().cache.pass)
            } else {
                Default::default()
            }
        )
    }
}

type Pos = usize;

pub type Slice = RangeInclusive<Pos>;

pub trait Slicable {
    fn slice(&self) -> Slice;
}

// trait ParseItem: Sized {
//     type Output = Result<Self, Diag>;
//     fn parse(arg: &mut ParseArgs) -> Self::Output;
// }

// trait Parse: ParseItem {
//     fn parse_item(arg: &mut ParseArgs) -> Self::Output;
//     fn check_good_cache(arg: &ParseArgs) -> Option<Self>;
// }

pub trait CommonTypes: Sized {
    const CONST: Construct;
    type Output = Result<Self, Self::Error>;
    type Error = Diag;
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token<T>(T, Slice);

impl<T> Slicable for Token<T> {
    fn slice(&self) -> Slice {
        self.1.clone()
    }
}

#[parse_test]
fn ident(print: Print) {
    let check = |c| {
        assert_eq!(
            Ident::recog(&mut (c, print.clone()).into(), 0).is_ok(),
            true
        );
    };
    check("sdfsfd");

    let check_err = |c, b| {
        assert_eq!(
            Ident::recog(&mut (c, print.clone()).into(), 0)
                .err()
                .unwrap()
                .error,
            b
        );
    };
    check_err("*", ErrorType::Ident(IdentError::StartsWithAlphabetic));
    check_err("!", ErrorType::Ident(IdentError::StartsWithAlphabetic));
}

pub trait CacheCheck: CommonTypes
where
    Self: Sized,
{
    const PREFIX: &str;

    fn check_pass(arg: &mut ParseArgs, l: usize) -> Option<(usize, Self)>
    where
        Self: Slicable,
    {
        let when_not_fail = arg.code.cursor;
        arg.c_a_d
            .borrow()
            .cache
            .pass
            .iter()
            .enumerate()
            .find_map(|(i, k)| {
                k.items.get(k.index).and_then(|v| {
                    (v.0 == Self::CONST && v.1 == when_not_fail).then(|| {
                        let v: Self = CacheCheck::unwrap_item(v.2.clone());
                        arg.code.cursor = v.slice().end() + 1;
                        arg.print.from_cache::<true>(Self::PREFIX, Self::CONST, l);
                        (i, v)
                    })
                })
            })
    }

    fn check_fail(arg: &mut ParseArgs, l: usize) -> Option<Diag> {
        arg.c_a_d
            .borrow()
            .cache
            .fails
            .get(&(Self::CONST, arg.code.cursor))
            .map(|e| {
                arg.print.from_cache::<false>(Self::PREFIX, Self::CONST, l);
                e
            })
            .cloned()
    }

    fn unwrap_item(item: ConstructItem) -> Self;
}

pub trait Recog: CacheCheck
where
    Self: Sized + Slicable,
{
    fn recog(arg: &mut ParseArgs, l: usize) -> Result<Self, Diag> {
        if let Some(e) = Self::check_fail(arg, l) {
            Err(e)
        } else {
            Self::check_pass(arg, l)
                .map(|(_, s)| Ok(s))
                .unwrap_or_else(|| Self::parse2(arg, l))
        }
    }

    fn parse2(arg: &mut ParseArgs, l: usize) -> Result<Self, Diag>;
}

trait TokenRecog<T>: CommonTypes + CacheCheck
where
    T: Sized,
{
    fn parse(arg: &mut ParseArgs, l: usize) -> Result<Token<T>, Diag> {
        let pos = arg.code.cursor;
        let mut fast_print = |arg: &mut ParseArgs, v| {
            arg.print.print_colored(
                format!("{v} {}", arg.get_head("token", Self::CONST, pos)),
                l,
            );
        };

        Self::consume_parse(arg)
            .map(|v| {
                fast_print(arg, format!("{}", tmp_pass_or_fail::<true>()));
                v
            })
            .map_err(|e| {
                fast_print(arg, format!("{}({})", tmp_pass_or_fail::<false>(), e.end()));
                e
            })
    }

    fn consume_parse(arg: &mut ParseArgs) -> Result<Token<T>, Diag> {
        Self::after_debug(arg).map(|v| {
            arg.code.cursor = v.1.end() + 1;
            v
        })
    }

    fn after_debug(arg: &ParseArgs) -> Result<Token<T>, Diag>;
}

trait EnumRecog: CommonTypes + CacheCheck {
    fn parse(arg: &mut ParseArgs, l: usize) -> Result<Self, Diag>;

    // есть необходимость в `consume` ведь мы делаем `arg.clone`
    fn consume_parse(arg: &mut ParseArgs, l: usize) -> Result<Self, Diag>;

    // enum не кешируется, потому что:
    // 1. состоит из токенов и конструкций, которые кешируются
    // 2. enum состоит из вариций, если кешировать одну это значит кешировать любую другую
    // fn cache_parse(arg: &mut ParseArgs) -> Self::Output
    fn after_debug(arg: &ParseArgs, l: usize) -> Result<Self, Diag>;
}

trait ConstructRecog: CommonTypes + CacheCheck
where
    Self: Sized,
{
    // нет необходимости в consume ведь `items` сами это делаеют
    fn parse(arg: &mut ParseArgs, l: usize) -> Result<Self, Diag> {
        arg.print
            .print_colored(arg.get_head("cons", Self::CONST, arg.code.cursor), l);
        Self::after_debug(arg, l)
            .map(|v| {
                arg.print.pass_or_fail::<true>(l);
                v
            })
            .map_err(|e| {
                arg.print.pass_or_fail::<false>(l);
                e
            })
    }

    fn after_debug(arg: &mut ParseArgs, l: usize) -> Result<Self, Diag>;
}

pub trait SequenceRecog: CommonTypes
where
    Self: Sized,
{
    type Item: Recog + Clone;

    fn recog(arg: &mut ParseArgs, l: usize) -> Self;

    fn items(arg: &mut ParseArgs, l: usize) -> Vec<Self::Item> {
        arg.print.print_tab(
            format!(
                "{} ignored",
                colored(arg.get_head("items", Self::CONST, arg.code.cursor), l)
            ),
            l,
        );
        let mut vec = vec![];
        loop {
            Self::join(arg, l + 1);

            if arg.code.cursor >= arg.code.source.len() {
                break;
            }
            match Self::Item::recog(&mut arg.clone(), l + 1) {
                Ok(v) => {
                    // не влияет на алгоритм, но очищает ненужную память, ускоряет поиск в списке
                    arg.c_a_d.borrow_mut().cache.pass.clear();
                    vec.push(v.clone());
                    arg.code.cursor = *v.slice().end() + 1;
                }
                Err(e) => match Self::break_(arg, l + 1, e) {
                    ControlFlow::Continue(..) => continue,
                    ControlFlow::Break(..) => break,
                },
            };
        }
        Self::join(arg, l + 1);
        if vec.is_empty() {
            arg.print.pass_or_fail::<false>(l);
        } else {
            arg.print.pass_or_fail::<true>(l);
        }
        vec
    }

    fn join(arg: &mut ParseArgs, l: usize) {}

    fn break_(arg: &mut ParseArgs, l: usize, e: Diag) -> ControlFlow<()> {
        let e = arg.c_a_d.borrow().clone().cache.check(e);
        arg.code.cursor = *e.end() + 1;
        arg.c_a_d.borrow_mut().errors.push(e);
        ControlFlow::Continue(())
    }
}

trait ConstructParse<const N: usize> {
    fn recog(&self, arg: &mut ParseArgs, l: usize) -> Result<[ConstructItem; N], Diag>;
}

trait ConstructMarker {
    fn item(&self, arg: &mut ParseArgs, l: usize) -> Result<ConstructItem, Diag>;
}

impl<const N: usize> ConstructParse<N> for [Box<dyn ConstructMarker>; N] {
    fn recog(&self, arg: &mut ParseArgs, l: usize) -> Result<[ConstructItem; N], Diag> {
        self.iter()
            .map(|item| item.item(arg, l))
            .collect::<Result<Vec<_>, _>>()
            .map(|v| v.try_into().unwrap())
    }
}

#[test]
fn items() {
    Items::recog(
        &mut r#"trait Lt {
    fn tmp()
}
impl Lt for main {
    fn tmp() {
        sdf = 2
    }
    const T = "dsf"
}
struct T { sdfds:T }
fn main ( sf sdf ) {
    sdf = 2
}
const T = "dsf""#
            .into(),
        0,
    );
}

#[test]
fn ignore() {
    let t = [Construct::Ignore, Construct::Ident]
        .recog(&mut " \n\t\t  sdfsdf".into(), 0)
        .unwrap();
    assert_eq!(t[1], ConstructItem::Ident(Ident::new(6..=11)));
}

/// для диганостики обрабтывает единичные символы, а не связку
fn reg_observe(arg: &ParseArgs, reg: &str) -> Result<Slice, usize> {
    Regex::new(&format!("^{reg}"))
        .unwrap()
        .find(&arg.code.get_residue())
        .map(|mat| arg.code.cursor + mat.start()..=arg.code.cursor + mat.end() - 1)
        .ok_or(arg.code.cursor)
}
