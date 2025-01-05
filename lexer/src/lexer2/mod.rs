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
mod print;
mod tests;

use crate::parse;
use cache_and_diags::{diag::Diag, Cache, CacheAndDiags, PassList};
use code::{Code, Source};
use colored::Colorize;
use macros::constructor;
use paste::paste;
use print::{tmp_pass_or_fail, Print};
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
    fmt::Display,
    hash::DefaultHasher,
    io::{self, stdout, Cursor, Read, Write},
    ops::RangeInclusive,
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

impl<'a> From<&'a str> for ParseArgs {
    fn from(value: &'a str) -> Self {
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

trait CommonTypes: Sized {
    const CONST: Construct;
    type Output = Result<Self, Self::Error>;
    type Error = Diag;
}

// pub struct Token<T> {
//     slice: Slice,
//     type_: T,
// }

constructor!(
    tokens {
        WhiteSpace r" +"
        NextLine r"\n"
        Tab r"\t"
        Ident [StartsWithNumber] {
            // r"\b[_a-zA-Z][_a-zA-Z0-9]*\b"
            let start_rule = |char: char| char.is_alphabetic() || char == '_';

            let mut iter = arg.code.iter();
            let &(i, char) = iter.next().ok_or((arg.code.cursor..=arg.code.cursor, ErrorType::LineOver))?;

            let s = (!start_rule(char)).then_some(i);
            let mut e = None;
            let start = i;

            let end = if start == arg.code.source.len() - 1 {
                start
            } else {
                iter.find_map(|&(i, char)| {
                    if start_rule(char) || char.is_digit(10) {
                        e = Some(i);
                        None
                    } else {
                        Some(i - 1)
                    }
                })
                .unwrap_or_else(|| {
                    e = Some(arg.code.source.len() - 1);
                    arg.code.source.len() - 1
                })
            };
            match (s, e) {
                (Some(s), Some(t)) => return Err((s..=t, ErrorType::Ident(IdentError::StartsWithNumber))),
                (Some(s), None) => return Err((s..=s, ErrorType::Ident(IdentError::StartsWithNumber))),
                _ => {}
            }

            Ok(start..=end)
        }
        Number r"\b\d+\b"
        String [StartsWithNumber StartsWithQuote EndsWithQuote] {
            // r#""[^"\\]*(?:\\.[^"\\]*)*""#
            let mut iter = arg.code.iter();

            let &(i, char) = iter.next().ok_or((arg.code.cursor..=arg.code.cursor, ErrorType::LineOver))?;
            let start = (char == '"').then_some(i).ok_or((i..=i, ErrorType::String(StringError::StartsWithQuote)))?;

            for &(i, char) in iter.clone() {
                if char == '"' {
                    return Ok(start..=i);
                }
            }

            let tmp = iter.last().unwrap().0;

            Err((tmp..=tmp, ErrorType::String(StringError::EndsWithQuote)))
        }
        Distribution r#"\.\."#
        NameSpace "::"
        OpenFigureBracket r#"\{"#
        CloseFigureBracket r#"}"#
        OpenRoundBracket r#"\("#
        CloseRoundBracket r#"\)"#
        Eq r#"="#
        Add r#"\+"#
        Sub r#"-"#
        Mul r"\*"
        Div r#"/"#

        Fn "fn"
        Const "const"
        Struct "struct"
        Trait "trait"
        Let "let"

        Comma ","
        Colon ":"
    }
    enums {
        Item -> FnHead | AnyBlock | AssignExpr | Literal | Idents | Ignore
        AnyBlock -> NamedDistrBlock | DistrBlock | NamedBlock | Block
        AssignExpr -> AssignAnd | Assign
        Literal -> String | Number

        Ignore -> WhiteSpace | NextLine | Tab

        Op -> Add | Sub | Mul | Div

        Idents -> Keyword | Ident
        Keyword -> Fn | Const | Struct | Trait | Let

        FnArgs -> StructArgsC | TupleArgsC
        StructArgsV -> StructArg | Ignore
        TupleArgsV -> TupleArg | Ignore

        CacheConstructItem -> Var1 | Var2
        CacheConstructHead -> Var3 | CommCons1
        CacheToken -> CommCons1 | Ident
        CacheEnum -> Var5 | Op
        CacheConstructWalkthroug -> Var6 | Var7 | Var8
    }
    constructs {
        NamedDistrBlock -> NamedBlock (Ignore) Distribution
        DistrBlock -> Ident (Ignore) Distribution
        NamedBlock -> Ident (Ignore) Block
        Block -> OpenFigureBracket Items CloseFigureBracket

        AssignAnd -> Ident (Ignore) OpEq (Ignore) Literal
        Assign -> Ident (Ignore) Eq (Ignore) Literal
        OpEq -> Op Eq

        FnHead -> Fn (Ignore) Ident (Ignore) FnArgs (Ignore) Block
        StructArgsC -> OpenFigureBracket (Ignore) StructArgsI (Ignore) CloseFigureBracket
        TupleArgsC -> OpenRoundBracket (Ignore) TupleArgsI (Ignore) CloseRoundBracket
        StructArg -> Ident (Ignore) Colon (Ignore) Ident (Ignore) Comma
        TupleArg -> Ident (Ignore) Comma
        Tmp -> Ident Eq

        CommCons1 -> Ident Distribution

        Var1 -> Ident Ignore Eq Number WhiteSpace NamedDistrBlock Item Ignore AssignAnd WhiteSpace AssignExpr Literal Sub OpEq Op Number
        Var2 -> Ident Ignore Eq Number WhiteSpace NamedDistrBlock Item Ignore AssignAnd WhiteSpace AssignExpr Literal Sub OpEq Op

        Var3 -> CommCons1 Ident

        Var5 -> Op Ident

        Var6 -> Ident CloseFigureBracket Distribution
        Var7 -> Ident CloseFigureBracket OpenFigureBracket CloseFigureBracket
        Var8 -> Ident CloseFigureBracket OpenFigureBracket WhiteSpace
    }
    items {
        Items(Item) CloseFigureBracket
        StructArgsI(StructArgsV) CloseFigureBracket
        TupleArgsI(TupleArgsV) CloseRoundBracket
    }
);

/// для диганостики обрабтывает единичные символы, а не связку
fn reg_observe(arg: &ParseArgs, reg: &str) -> Result<Slice, usize> {
    Regex::new(&format!("^{reg}"))
        .unwrap()
        .find(&arg.code.get_residue())
        .map(|mat| arg.code.cursor + mat.start()..=arg.code.cursor + mat.end() - 1)
        .ok_or(arg.code.cursor)
}
