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

trait CommonTypes: Sized {
    const CONST: Construct;
    type Output = Result<Self, Self::Error>;
    type Error = Diag;
}

// pub struct Token<T> {
//     slice: Slice,
//     type_: T,
// }

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

constructor!(
    tokens {
        Ident [StartsWithAlphabetic Alphabetic] {
            // r"\b[_a-zA-Z][_a-zA-Z0-9]*\b"
            let start_rule = |char: char| char.is_alphabetic() || char == '_';

            let mut iter = arg.code.iter();
            let &(i, char) = iter.next().ok_or((arg.code.cursor..=arg.code.cursor, ErrorType::LineOver))?;

            if let Some(start) = start_rule(char).then_some(i) {
                if start == arg.code.source.len() - 1 {
                    Ok(start..=start)
                } else {
                    let end = iter.find_map(|&(i, char)| {
                            (!(start_rule(char) || char.is_digit(10))).then_some(i-1)
                        })
                        .unwrap_or_else(|| {
                            arg.code.source.len() - 1
                        });
                    Ok(start..=end)
                }
            } else {
                if i == arg.code.source.len() - 1 {
                    Err((i..=i, ErrorType::Ident(IdentError::StartsWithAlphabetic)))
                } else {
                    let end = iter.find_map(|&(i, char)| {
                            (!( start_rule(char) || char.is_digit(10))).then_some(i - 1)
                        })
                        .unwrap_or_else(|| {
                            arg.code.source.len() - 1
                        });
                    Err((i..=end, ErrorType::Ident(IdentError::Alphabetic)))
                }
            }
        }
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
    }
    items {
        Items ! (Item) (Ignore)
    }
    common {
        Item -> FnC | StructC | TraitC | ImplV | AnyBlock | ConstC | AssignExpr | Literal | Ident
            TraitC (Ignore) -> Trait Ident MethodsC
                MethodsC (Ignore) -> OpenFigureBracket MethodsI CloseFigureBracket
                    MethodsI ! (FnHead) (Ignore) CloseFigureBracket
            ImplV -> ImplFor | ImplC
                ImplFor (Ignore) -> Impl Ident For Ident ImplItemsC
                ImplC (Ignore) -> Impl Ident ImplItemsC
                    ImplItemsC (Ignore) -> OpenFigureBracket ImplItemsI CloseFigureBracket
                        ImplItemsI ! (ImplItemsV) (Ignore) CloseFigureBracket
                            ImplItemsV -> ConstC | FnC
            StructC (Ignore) -> Struct Ident Args
            FnC (Ignore) -> FnHead Block
                FnHead (Ignore) -> Fn Ident Args
            AnyBlock -> NamedDistrBlock | DistrBlock | NamedBlock | Block
                NamedDistrBlock (Ignore) -> NamedBlock Distribution
                DistrBlock (Ignore) -> Path Distribution
                NamedBlock (Ignore) -> Ident Block
                Block -> OpenFigureBracket BlockItems CloseFigureBracket
                    BlockItems ! (Item) (Ignore) CloseFigureBracket
            ConstC (Ignore) -> Const Assign
            AssignExpr -> AssignAnd | Assign
                AssignAnd (Ignore) -> IdentAndType OpEq Literal
                    OpEq -> Op Eq
                Assign (Ignore) -> IdentAndType Eq Literal
            Literal -> String | Number
        Args -> StructArgsC | TupleType
            StructArgsC (Ignore) -> OpenFigureBracket StructArgsI CloseFigureBracket
                StructArgsI ! (IdentAndTypeC) (Ignore) CloseFigureBracket
        Ignore (IgnoreV) #
            IgnoreV -> WhiteSpace | NextLine | Tab
                WhiteSpace r" +"
                NextLine r"\n"
                Tab r"\t"
        Op -> Add | Sub | Mul | Div
            Add r#"\+"#
            Sub r#"-"#
            Mul r"\*"
            Div r#"/"#
        IdentAndType -> IdentAndTypeC | Ident
            IdentAndTypeC (Ignore) -> Ident Colon Type
        Type -> TupleType | BaseType
            TupleType (Ignore) -> OpenRoundBracket TupleTypeI CloseRoundBracket
                TupleTypeI ! (Type) (Ignore) CloseRoundBracket
            BaseType -> AnnotededTypeC | Ident
                AnnotededTypeC (Ignore) -> Ident OpenAngleBracket AnnotededTypeI CloseAngleBracket
                    OpenAngleBracket "<"
                    CloseAngleBracket ">"
                    AnnotededTypeI ! (AnnotededType) (Ignore) CloseAngleBracket
                        AnnotededType -> EqType | Ident
                            EqType (Ignore) -> Ident Eq Type!
        Path -> CurrenPath | EndPath | Ident
            CurrenPath (Ignore) -> CurrentPathV EndPath
                CurrentPathV -> Self_ | Super | Crate | Ident
                EndPath -> WithItemsEnd | IdentPath
                    IdentPath ! (PathEl) #
                            PathEl (Ignore)-> NameSpace Ident
                    WithItemsEnd (Ignore) ->  IdentPath NameSpace PathItemEnd
                        PathItemEnd -> GlobImport | PathItemsC
                            PathItemsC (Ignore) -> OpenFigureBracket PathItemsI CloseFigureBracket
                                PathItemsI ! (PathItemC) (Ignore) CloseFigureBracket
                                    PathItemC (Ignore) -> PathItemV Comma
                                        PathItemV -> Self_ | Super | GlobImport | Ident
                                            GlobImport r#"\*"#
        // Keywords
        Fn "fn"
        Const "const"
        Struct "struct"
        Trait "trait"
        Let "let"
        Impl "impl"
        Crate "crate"
        For "for"
        Self_ "self"
        Super "super"
    }
    common {
        Tmp -> Ident Eq
    }
    common {
        CacheConstructItem -> Var1 | Var2
        Var1 -> Ident Ignore Eq Number WhiteSpace NamedDistrBlock Item Ignore AssignAnd WhiteSpace AssignExpr Literal Sub OpEq Op Number
        Var2 -> Ident Ignore Eq Number WhiteSpace NamedDistrBlock Item Ignore AssignAnd WhiteSpace AssignExpr Literal Sub OpEq Op
    }
    common {
        CacheConstructHead -> Var3 | CommCons1
        CacheToken -> CommCons1 | Ident
        CacheEnum -> Var5 | Op
        CacheConstructWalkthroug -> Var6 | Var7 | Var8

        CommCons1 -> Ident Distribution
        Var3 -> CommCons1 Ident
        Var5 -> Op Ident
        Var6 -> Ident CloseFigureBracket Distribution
        Var7 -> Ident CloseFigureBracket OpenFigureBracket CloseFigureBracket
        Var8 -> Ident CloseFigureBracket OpenFigureBracket WhiteSpace
    }
);

trait CacheCheck: CommonTypes
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

trait Recog: CacheCheck
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

trait TokenRecog: CommonTypes + CacheCheck
where
    Self: Sized,
{
    fn parse(arg: &mut ParseArgs, l: usize) -> Result<Self, Diag> {
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

    fn consume_parse(arg: &mut ParseArgs) -> Result<Self, Diag> {
        Self::after_debug(arg).map(|(v, slice)| {
            arg.code.cursor = slice.end() + 1;
            v
        })
    }

    fn after_debug(arg: &ParseArgs) -> Result<(Self, RangeInclusive<usize>), Diag>;
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
    assert_eq!(t[1], ConstructItem::Ident(Ident(6..=11)));
}

/// для диганостики обрабтывает единичные символы, а не связку
fn reg_observe(arg: &ParseArgs, reg: &str) -> Result<Slice, usize> {
    Regex::new(&format!("^{reg}"))
        .unwrap()
        .find(&arg.code.get_residue())
        .map(|mat| arg.code.cursor + mat.start()..=arg.code.cursor + mat.end() - 1)
        .ok_or(arg.code.cursor)
}
