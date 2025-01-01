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

mod test;

use colored::Colorize;
use macros::constructor;
use paste::paste;
use regex::Regex;
use regex_automata::{
    dfa::{dense::DFA, Automaton},
    nfa::thompson::NFA,
    util::start::Config,
    Input,
};
use regex_syntax::{ast::parse::Parser as AstParser, Parser};
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
use tracing::info;
#[derive(Clone, Debug)]
pub struct ParseArgs {
    code: Code,
    pub c_a_d: Arc<RefCell<CacheAndDiags>>,
}

impl<'a> From<&'a str> for ParseArgs {
    fn from(value: &'a str) -> Self {
        Self::new(&value)
    }
}

impl ParseArgs {
    fn new(source: &str) -> Self {
        Self {
            code: Code {
                source: S {
                    real_source: source.into(),
                    source: Arc::new(source.chars().enumerate().collect()),
                },
                cursor: Default::default(),
            },
            c_a_d: Default::default(),
        }
    }
}
type Source = Arc<Vec<(usize, char)>>;

#[derive(Clone, Debug)]
struct Code {
    source: S,
    cursor: Pos,
}

impl<'a> IntoIterator for &'a Code {
    type Item = &'a (usize, char);
    type IntoIter = std::slice::Iter<'a, (usize, char)>;

    fn into_iter(self) -> Self::IntoIter {
        self.source[self.cursor..].iter()
    }
}

impl Code {
    fn get_current(&self) -> (usize, char) {
        self.source[self.cursor]
    }
    fn t(&self) -> std::string::String {
        self.source
            .as_ref()
            .into_iter()
            .skip_while(|&&(i, _)| i < self.cursor)
            .map(|(_, v)| v)
            .collect()
    }
    fn len(&self) -> usize {
        self.source.len()
    }
    fn iter(&self) -> std::slice::Iter<'_, (usize, char)> {
        self.source[self.cursor..].into_iter()
    }
}

type Pos = usize;

#[derive(Clone, Debug, Deref)]
pub struct S {
    pub real_source: std::string::String,
    #[deref]
    source: Source,
}

// impl S {
//     fn new(source: Source) -> Self {
//         Self {
//             real_source: source.clone().iter().map(|(_, v)| v).collect(),
//             source,
//         }
//     }
// }

#[derive(Clone, Default, Debug)]
pub struct CacheAndDiags {
    cursor: Option<Pos>,
    cache: Cache,
    pub errors: Vec<Diag>,
    warnings: PosS<Vec<Construct>>,
}

#[derive(Clone, Default, Debug)]
struct Cache {
    // записываем только в const_item
    pass: Vec<PassList>,
    fails: HashMap<(Construct, Pos), Diag>,
}

#[derive(Clone, Default, Debug, Deref)]
struct PassList {
    index: usize,
    #[deref]
    items: Vec<Pass>,
}
type Pass = (Construct, Pos, ConstructItem);
impl PassList {
    fn new(items: Vec<Pass>) -> Self {
        Self {
            index: Default::default(),
            items,
        }
    }
    fn get(&self) -> &(Construct, Pos, ConstructItem) {
        &self.items[self.index]
    }
}

#[derive(Clone, Debug, Deref)]
pub struct Diag {
    #[deref]
    pub slice: Slice,
    pub source: S,
    pub error: ErrorType,
}

impl std::fmt::Display for Diag {
    fn fmt(&self, f_: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Diag {
            source: S {
                real_source: source,
                ..
            },
            slice,
            error,
        } = self;
        let get_line = |pos| {
            let mut acc = 0;
            source
                .split_inclusive('\n')
                .enumerate()
                .find_map(|(i, str)| {
                    acc += str.chars().count();
                    (pos < acc).then_some(i + 1)
                })
                .unwrap()
        };
        let f = |v: &[char]| v.iter().collect::<std::string::String>();

        let source = source.chars().collect::<Vec<_>>();
        let (l, b, [min, max]) = ("|".blue(), "...".blue(), [10, 4]);

        let code = format!(
            "{}{}{}",
            f(&source[{
                let i = *slice.start();
                let r = if i < min { 0 } else { i - min };
                r..i
            }]),
            f(&source[slice.clone()]).underline().red(),
            f(&source[{
                let i = slice.end();
                i + 1..if source.len() - 1 - i < max {
                    source.len()
                } else {
                    i + max
                }
            }])
        );
        let front_p = 3;
        let f = " ".repeat(front_p);

        writeln!(
            f_,
            "
{f}{l}
{}{l} {b}{code}{}
{f}{l} {}{}
",
            format!("{:width$} ", get_line(*slice.end()), width = front_p - 1),
            if source.len() - 1 - slice.end() < max {
                Default::default()
            } else {
                format!("{b}")
            },
            " ".repeat(min + b.chars().count()),
            format!(
                "{}-Ожидается {error:?}",
                "^".repeat(slice.end() - slice.start() + 1),
            )
            .red()
        )
    }
}

type Warnings = PosS<Vec<Construct>>;

#[derive(Clone, Debug)]
struct PosS<T> {
    cursor: Option<Pos>,
    data: T,
}

impl<T> Default for PosS<Vec<T>> {
    fn default() -> Self {
        Self {
            cursor: Default::default(),
            data: Default::default(),
        }
    }
}

pub type Slice = RangeInclusive<Pos>;

pub trait Slicable {
    fn slice(&self) -> Slice;
}

trait ParseItem: Sized {
    type Output = Result<Self, Diag>;
    fn parse(arg: &mut ParseArgs) -> Self::Output;
}

trait Parse: ParseItem {
    fn parse_item(arg: &mut ParseArgs) -> Self::Output;
    fn check_good_cache(arg: &ParseArgs) -> Option<Self>;
}

trait CommonTypes: Sized {
    const CONST: Construct;
    type Output = Result<Self, Self::Error>;
    type Error = Diag;
}

/// для диганостики обрабтывает единичные символы, а не связку
fn reg_observe(arg: &ParseArgs, reg: &str) -> Result<Slice, usize> {
    Regex::new(&format!("^{reg}"))
        .unwrap()
        .find(&arg.code.t())
        .map(|mat| arg.code.cursor + mat.start()..=arg.code.cursor + mat.end() - 1)
        .ok_or(arg.code.cursor)
}

pub struct Token<T> {
    slice: Slice,
    type_: T,
}

fn from_cache<const PASS: bool>(pref: &str, c: Construct, l: usize) {
    print_tab(
        format!("{} {pref} {:?} from Cache", pass_or_fail::<PASS>(), c),
        l,
    );
}

fn pass_or_fail<const PASS: bool>() -> impl Display {
    if PASS {
        "✅ Pass"
    } else {
        "❌ Fail"
    }
}

fn print_colored(t: impl Display, l: usize) {
    print_tab(colored(t, l), l);
}

fn print_tab(t: impl Display, l: usize) {
    info!("{}{t}", tab(l));
    // println!("{}{t}", tab(l));
}

fn colored(t: impl Display, l: usize) -> std::string::String {
    let h = ((l * 47) % 360) as f32; // Разнообразие угла оттенка
    let s = 1.0; // Макс насыщенность
    let v = 1.0;
    let (r, g, b) = hsv_to_rgb(h, s, v);
    format!("{}", t.to_string().truecolor(r, g, b))
}

fn tab(l: usize) -> std::string::String {
    if l == 0 {
        Default::default()
    } else {
        (0..l).map(|i| colored("|  ", l - i - 1)).rev().collect()
    }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = match h as u32 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    let (r, g, b) = ((r + m) * 255.0, (g + m) * 255.0, (b + m) * 255.0);
    (r as u8, g as u8, b as u8)
}

constructor!(
    tokens {
        WhiteSpace r" +"
        NextLine r"\n"
        Tab r"\t"
        Ident [StartsWithNumber] {
            // r"\b[_a-zA-Z][_a-zA-Z0-9]*\b"
            let code = &mut arg.code.source.clone();

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
                _=>{}
            }

            Ok(start..=end)
        }
        Number r"\b\d+\b"
        String [StartsWithNumber StartsWithQuote EndsWithQuote] {
            // r#""[^"\\]*(?:\\.[^"\\]*)*""#
            let code = &mut arg.code.clone();

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
        OpenBracket r#"\{"#
        CloseBracket r#"}"#
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
    }
    enums {
        Item -> FnHeader | AnyBlock | AssignExpr | Literal | Idents | Ignore
        AnyBlock -> NamedDistrBlock | DistrBlock | NamedBlock | Block
        AssignExpr -> AssignAnd | Assign
        Literal -> String | Number

        Ignore -> WhiteSpace | NextLine | Tab

        Op -> Add | Sub | Mul | Div

        Idents -> Keyword | Ident
        Keyword -> Fn | Const | Struct | Trait | Let

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
        Block -> OpenBracket Items CloseBracket

        AssignAnd -> Ident (Ignore) OpEq (Ignore) Literal
        Assign -> Ident (Ignore) Eq (Ignore) Literal
        OpEq -> Op Eq

        BracketArgs -> OpenRoundBracket Args CloseRoundBracket
        Arg -> Ident Comma
        FnHeader -> Fn (Ignore) Ident (Ignore) BracketArgs (Ignore) Block

        Tmp -> Ident Eq

        CommCons1 -> Ident Distribution

        Var1 -> Ident Ignore Eq Number WhiteSpace NamedDistrBlock Item Ignore AssignAnd WhiteSpace AssignExpr Literal Sub OpEq Op Number
        Var2 -> Ident Ignore Eq Number WhiteSpace NamedDistrBlock Item Ignore AssignAnd WhiteSpace AssignExpr Literal Sub OpEq Op

        Var3 -> CommCons1 Ident

        Var5 -> Op Ident

        Var6 -> Ident CloseBracket Distribution
        Var7 -> Ident CloseBracket OpenBracket CloseBracket
        Var8 -> Ident CloseBracket OpenBracket WhiteSpace
    }
    items {
        Items(Item)
        Args(Arg)
    }
);
