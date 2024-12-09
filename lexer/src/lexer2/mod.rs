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

use colored::Colorize;
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

#[derive(Clone, Debug)]
pub struct ParseArgs {
    code: Code,
    pub c_a_d: Rc<RefCell<CacheAndDiags>>,
}

impl ParseArgs {
    fn new(source: &str) -> Self {
        Self {
            code: Code {
                source: source.chars().enumerate().collect(),
                cursor: Default::default(),
            },
            c_a_d: Default::default(),
        }
    }
}

#[derive(Clone, Debug)]
struct Code {
    source: Vec<(usize, char)>,
    cursor: Pos,
}

impl Code {
    fn get_current(&self) -> (usize, char) {
        self.source[self.cursor]
    }
    fn t(&self) -> std::string::String {
        self.source
            .clone()
            .into_iter()
            .skip_while(|&(i, _)| i < self.cursor)
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

#[derive(Clone, Default, Debug)]
pub struct CacheAndDiags {
    cursor: Option<Pos>,
    cache: Cache,
    pub errors: Vec<Error_>,
    warnings: PosS<Vec<Construct>>,
}

#[derive(Clone, Default, Debug)]
struct Cache {
    // записываем только в const_item
    pass: Vec<PassList>,
    fails: HashMap<(Construct, Pos), Error_>,
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

pub type Error_ = (Slice, Construct);
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

type Slice = RangeInclusive<Pos>;

trait Slicable {
    fn get_slice(&self) -> Slice;
}

trait ParseItem: Sized {
    type Output = Result<Self, Error_>;
    fn parse(arg: &mut ParseArgs) -> Self::Output;
}

trait Parse: ParseItem {
    fn parse_item(arg: &mut ParseArgs) -> Self::Output;
    fn check_good_cache(arg: &ParseArgs) -> Option<Self>;
}

macro_rules! tokens {
    ($cons_item:ident $arg:ident -> $t:literal) => {
        reg_observe($arg, $t).map(Self).map_err(|v| (v..=v, Construct::$cons_item))
    };
    ($cons_item:ident $arg:ident -> $($t:tt)*) => {
        ($($t)*)($arg).map_err(|i| (i, Construct::$cons_item))
    }
}

/// для диганостики обрабтывает единичные символы, а не связку
fn reg_observe(arg: &ParseArgs, reg: &str) -> Result<Slice, usize> {
    Regex::new(&format!("^{reg}"))
        .unwrap()
        .find(&arg.code.t())
        .map(|mat| arg.code.cursor + mat.start()..=arg.code.cursor + mat.end() - 1)
        .ok_or(arg.code.cursor)
}

macro_rules! k {
    (Items $( $ignore:ident )?) => {
        |arg, l, ptr, cache_if_error: &mut Vec<(Construct, Pos, ConstructItem)>, when_not_fail| {
            let v =  Items::recog(arg, l + 1);
            cache_if_error.push((
                Construct::Items,
                when_not_fail,
                ConstructItem::Items(v.clone()),
            ));
            $( $ignore::recog(arg, l + 1); )?
            Ok(v)
        }
    };
    ($cons_item:ident $( $ignore:ident )?) => {
        |arg: &mut ParseArgs, l: usize, ptr: &mut Option<usize>, cache_if_error: &mut Vec<(Construct, Pos, ConstructItem)>, when_not_fail| {
            let v = match $cons_item::check_pass(arg, l + 1) {
                Some((i, v)) => {
                    if let Some(v) = *ptr {
                        if v != i {
                            *ptr = None
                        }
                    } else {
                        *ptr = Some(i);
                    }

                    arg.c_a_d.borrow_mut().cache.pass[i].index += 1;
                    $( $ignore::recog(arg, l + 1); )?
                    v
                }
                None => {
                    match $cons_item::parse(arg, l + 1) {
                        Ok(v) => {
                            cache_if_error.push((
                                Construct::$cons_item,
                                when_not_fail,
                                ConstructItem::$cons_item(v.clone()),
                            ));
                            $( $ignore::recog(arg, l + 1); )?
                            v
                        }
                        Err(e) =>  {
                            if !cache_if_error.is_empty() {
                                if let Some(i) = *ptr {
                                    let v = &mut arg.c_a_d.borrow_mut().cache.pass[i];
                                    v.index = 0;
                                    v.items.extend(cache_if_error.clone());
                                } else {
                                    arg.c_a_d.borrow_mut().cache.pass.push(PassList::new(cache_if_error.clone()));
                                }
                            }
                            arg.c_a_d.borrow_mut().cache.fails.insert((Construct::$cons_item, when_not_fail), e.clone());
                            return Err(e);
                        }
                    }
                }
            };
            Ok(v)
        }
    };
}

macro_rules! m {
    (
        items {$(
            $items:ident
        )*}
        tokens {$(
            $init_item:ident $( { $($t:tt)* } )?
        )+}
        enums {$(
            $enum_name:ident -> $enum_item1:ident $(, $( $enum_item:ident ),+)?
        )*}
        constructs {$(
            $cons_name:ident -> $( $cons_item:ident $( ($ignore:ident) )?),+ [$n:tt]
        )*}
    ) => {
        $($(
            #[derive(Clone, Debug)]
            struct $init_item(Slice);
            impl $init_item {
                type Output = Result<Self, Error_>;

                fn recog(arg: &mut ParseArgs, l: usize) -> Self::Output {
                    if let Some(e) = Self::check_fail(arg, l) {
                        Err(e)
                    } else {
                        Self::check_pass(arg, l).map(|(_, s)| Ok(s)).unwrap_or_else(|| Self::parse(arg, l))
                    }
                }

                fn check_pass(arg: &mut ParseArgs, l: usize) -> Option<(usize, Self)> {
                    let when_not_fail = arg.code.cursor;
                    arg.c_a_d.borrow().cache.pass.iter().enumerate().find_map(|(i, k)| {
                        k.items.get(k.index).and_then(|v| {
                            (v.0 == Construct::$init_item && v.1 == when_not_fail).then(|| {
                                let ConstructItem::$init_item(ref v) = v.2 else {unreachable!()};
                                arg.code.cursor = v.get_slice().end() + 1;
                                from_cache::<true>("token", Construct::$init_item, l);
                                (i, v.clone())
                            })
                        })
                    })
                }

                fn check_fail(arg: &mut ParseArgs, l: usize) -> Option<Error_> {
                    arg.c_a_d.borrow().cache.fails.get(&(Construct::$init_item, arg.code.cursor)).map(|e| {
                        from_cache::<false>("token", Construct::$init_item, l);
                        e
                    }).cloned()
                }

                fn parse(arg: &mut ParseArgs, l: usize) -> Self::Output {
                    Self::consume_parse(arg)
                    .map(|v| {
                        println!("{}{} {}", tab(l), colored(pass_or_fail::<true>(), l), colored(format!("token {:?} {:?} {}", Construct::$init_item, arg.c_a_d.borrow().cache.pass, arg.code.cursor), l));
                        v
                    }).map_err(|e| {
                        println!("{}{} {}", tab(l), colored(pass_or_fail::<false>(), l), colored(format!("token {:?} {:?} {}", Construct::$init_item, arg.c_a_d.borrow().cache.pass, arg.code.cursor), l));
                        e
                    })
                }

                fn consume_parse(arg: &mut ParseArgs) -> Self::Output {
                    Self::after_debug(arg)
                    .map(|v| {
                        arg.code.cursor = v.get_slice().end() + 1;
                        v
                    })
                }

                fn after_debug(arg: &ParseArgs) -> Self::Output {
                    tokens!($init_item arg -> $($t)*)
                }
            }

            impl Slicable for $init_item {
                fn get_slice(&self) -> Slice {
                    self.0.clone()
                }
            }
        )?)+

        $(
            #[derive(Clone, Debug)]
            enum $enum_name {
                $enum_item1($enum_item1),
                $(  $( $enum_item($enum_item) ),+ )?
            }

            impl $enum_name {
                type Output = Result<Self, Error_>;
                fn recog(arg: &mut ParseArgs, l: usize) -> Self::Output {
                    if let Some(e) = Self::check_fail(arg, l) {
                        Err(e)
                    } else {
                        Self::check_pass(arg, l).map(|(_, s)| Ok(s)).unwrap_or_else(|| Self::parse(arg, l))
                    }
                }

                fn check_pass(arg: &mut ParseArgs, l: usize) -> Option<(usize, Self)> {
                    let when_not_fail = arg.code.cursor;
                    arg.c_a_d.borrow().cache.pass.iter().enumerate().find_map(|(i, k)| {
                        k.items.get(k.index).and_then(|v| {
                            (v.0 == Construct::$enum_name && v.1 == when_not_fail).then(|| {
                                let ConstructItem::$enum_name(ref v) = v.2 else {unreachable!()};
                                arg.code.cursor = v.get_slice().end() + 1;
                                from_cache::<true>("enum", Construct::$enum_name, l);
                                (i, v.clone())
                            })
                        })
                    })
                }

                fn check_fail(arg: &mut ParseArgs, l: usize) -> Option<Error_> {
                    arg.c_a_d.borrow().cache.fails.get(&(Construct::$enum_name, arg.code.cursor)).map(|e| {
                        from_cache::<false>("enum", Construct::$enum_name, l);
                        e
                    }).cloned()
                }

                fn parse(arg: &mut ParseArgs, l: usize) -> Self::Output {
                    print_colored(format!("enum {:?} {:?} {}", Construct::$enum_name, arg.c_a_d.borrow().cache.pass, arg.code.cursor), l);
                    Self::consume_parse(arg, l).map(|v| {
                        print_colored(pass_or_fail::<true>(), l);
                        v
                    }).map_err(|e| {
                        print_colored(pass_or_fail::<false>(), l);
                        e
                    })
                }

                // есть необходимость в `consume` ведь мы делаем `arg.clone`
                fn consume_parse(arg: &mut ParseArgs, l: usize) -> Self::Output {
                    Self::after_debug(arg, l)
                    .map(|v| {
                        arg.code.cursor = v.get_slice().end() + 1;
                        v
                    })
                }

                // enum не кешируется, потому что:
                // 1. состоит из токенов и конструкций, которые кешируются
                // 2. enum состоит из вариций, если кешировать одну это значит кешировать любую другую
                // fn cache_parse(arg: &mut ParseArgs) -> Self::Output
                fn after_debug(arg: &ParseArgs, l: usize) -> Self::Output {
                    let mut error: Option<Error_> = None;

                    match $enum_item1::recog(&mut arg.clone(), l + 1).map(Self::$enum_item1) {
                        Ok(v) => return Ok(v),
                        Err(e) =>  {
                            match error {
                                Some(v) if e.0.end() > v.0.end() => error = Some(e.clone()),
                                None => error = Some(e.clone()),
                                _ => {}
                            };
                            $($(
                                match $enum_item::recog(&mut arg.clone(), l + 1).map(Self::$enum_item) {
                                    Ok(v) => return Ok(v),
                                    Err(e) =>  {
                                        match error {
                                            Some(v) if e.0.end() > v.0.end() => error = Some(e.clone()),
                                            None => error = Some(e.clone()),
                                            _ => {}
                                        };
                                    }
                                }
                            )+)?
                        }
                    }

                    Err(error.clone().unwrap())
                }
            }

            impl Slicable for $enum_name {
                fn get_slice(&self) -> Slice {
                    match self {
                        Self::$enum_item1(v) => v.get_slice(),
                        $(  $( Self::$enum_item(v) => v.get_slice() ),+ )?
                    }
                }
            }
        )*

        $(
            #[derive(Clone, Debug)]
            struct $cons_name($( $cons_item ),+);
            impl $cons_name {
                type Output = Result<Self, Error_>;

                fn recog(arg: &mut ParseArgs, l: usize) -> Self::Output {
                    if let Some(e) = Self::check_fail(arg, l) {
                        Err(e)
                    } else {
                        Self::check_pass(arg, l).map(|(_, s)| Ok(s)).unwrap_or_else(|| Self::parse(arg, l))
                    }
                }

                fn check_pass(arg: &mut ParseArgs, l: usize) -> Option<(usize, Self)> {
                    let when_not_fail = arg.code.cursor;
                    arg.c_a_d.borrow().cache.pass.iter().enumerate().find_map(|(i, k)| {
                        k.items.get(k.index).and_then(|v| {
                            (v.0 == Construct::$cons_name && v.1 == when_not_fail).then(|| {
                                let ConstructItem::$cons_name(ref v) = v.2 else {unreachable!()};
                                arg.code.cursor = v.get_slice().end() + 1;
                                from_cache::<true>("cons", Construct::$cons_name, l);
                                (i, v.clone())
                            })
                        })
                    })
                }

                fn check_fail(arg: &mut ParseArgs, l: usize) -> Option<Error_> {
                    arg.c_a_d.borrow().cache.fails.get(&(Construct::$cons_name, arg.code.cursor)).map(|e| {
                        from_cache::<false>("cons", Construct::$cons_name, l);
                        e
                    }).cloned()
                }

                // нет необходимости в consume ведь `items` сами это делаеют
                fn parse(arg: &mut ParseArgs, l: usize) -> Self::Output {
                    print_colored(format!("cons {:?} {:?} {}", Construct::$cons_name, arg.c_a_d.borrow().cache.pass, arg.code.cursor), l);
                    Self::after_debug(arg, l).map(|v| {
                        print_colored(pass_or_fail::<true>(), l);
                        v
                    }).map_err(|e| {
                        print_colored(pass_or_fail::<false>(), l);
                        e
                    })
                }

                fn after_debug(arg: &mut ParseArgs, l: usize) -> Self::Output {
                    let mut cache_if_error: Vec<(Construct, Pos, ConstructItem)> = Default::default();
                    let mut ptr = Default::default();
                    Ok(
                        Self(
                            $(
                                {
                                    let when_not_fail = arg.code.cursor;
                                    k!($cons_item $( $ignore )?)(arg, l, &mut ptr, &mut cache_if_error, when_not_fail)?
                                }
                            ),+
                        )
                    )
                }
            }

            impl Slicable for $cons_name {
                fn get_slice(&self) -> Slice {
                    let start = self.0.get_slice();
                    let end = self.$n.get_slice();
                    *start.start()..=*end.end()
                }
            }
        )*

        #[derive(Clone, Debug, Eq, PartialEq, Hash)]
        pub enum Construct {
            $($items),+,
            $($init_item),+,
            $($enum_name),+,
            $($cons_name),*
        }

        #[derive(Clone, Debug)]
        enum ConstructItem {
            $($items($items)),+,
            $($init_item($init_item)),+,
            $($enum_name($enum_name)),+,
            $($cons_name($cons_name)),*
        }

        trait ConstructParse<const N: usize> {
            fn recog(&self, arg: &mut ParseArgs, l: usize) -> Result<[ConstructItem; N], Error_>;
        }
        impl<const N: usize> ConstructParse<N> for [Construct; N] {
            fn recog(&self, arg: &mut ParseArgs, l: usize) -> Result<[ConstructItem; N], Error_> {
                self.iter().map(|item| {
                    match item {
                        $(Construct::$items => {
                            Ok(ConstructItem::$items($items::recog(arg, l)))
                        }),+,
                        $(Construct::$init_item => {
                            $init_item::recog(arg, l).map(|v| ConstructItem::$init_item(v))
                        }),+,
                        $(Construct::$enum_name => {
                            $enum_name::recog(arg, l).map(|v| ConstructItem::$enum_name(v))
                        }),+,
                        $(Construct::$cons_name => {
                            $cons_name::recog(arg, l).map(|v| ConstructItem::$cons_name(v))
                        }),*
                    }
                }).collect::<Result<Vec<_>, _>>().map(|v| v.try_into().unwrap())
            }
        }

        impl Slicable for ConstructItem {
            fn get_slice(&self) -> Slice {
                match self {
                    $(Self::$items(v) => v.get_slice()),+,
                    $(Self::$init_item(v) => v.get_slice()),+,
                    $(Self::$enum_name(v) => v.get_slice()),+,
                    $(Self::$cons_name(v) => v.get_slice()),*
                }
            }
        }
    };
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
    let h = ((l * 47) % 360) as f32; // Разнообразие угла оттенка
    let s = 1.0; // Макс насыщенность
    let v = 1.0;
    let (r, g, b) = hsv_to_rgb(h, s, v);
    println!("{}{}", tab(l), colored(t, l));
}

fn colored(t: impl Display, l: usize) -> std::string::String {
    let h = ((l * 47) % 360) as f32; // Разнообразие угла оттенка
    let s = 1.0; // Макс насыщенность
    let v = 1.0;
    let (r, g, b) = hsv_to_rgb(h, s, v);
    format!("{}", t.to_string().truecolor(r, g, b))
}

fn print_tab(t: impl Display, l: usize) {
    println!("{}{t}", tab(l));
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

m!(
    items {
        Items
    }
    tokens {
        WhiteSpace {r" +"}
        NextLine {r"\n"}
        Tab {r"\t"}
        Ident {
            // r"\b[_a-zA-Z][_a-zA-Z0-9]*\b"
            |arg: &ParseArgs| {
                let code = &mut arg.code.source.clone();

                let start_rule = |char: char| char.is_alphabetic() || char == '_';

                let mut iter = arg.code.iter();
                let &(i, char) = iter.next().ok_or(arg.code.cursor..=arg.code.cursor)?;
                let start = start_rule(char).then_some(i).ok_or(i..=i)?;

                let end = if start == arg.code.source.len() - 1 {
                    start
                } else {
                    iter.find_map(|&(i, char)| {
                        if start_rule(char) || char.is_digit(10) {
                            None
                        } else {
                            Some(i - 1)
                        }
                    })
                    .unwrap_or(arg.code.source.len() - 1)
                };
                Ok(Self(start..=end))
            }
        }
        Number {r"\b\d+\b"}
        String {
            // r#""[^"\\]*(?:\\.[^"\\]*)*""#
            |arg: &ParseArgs| {
                let code = &mut arg.code.clone();

                let mut iter = arg.code.iter();

                let &(i, char) = iter.next().ok_or(arg.code.cursor..=arg.code.cursor)?;
                let start = (char == '"').then_some(i).ok_or(i..=i)?;

                for &(i, char) in iter.clone() {
                    if char == '"' {
                        return Ok(Self(start..=i));
                    }
                }
                let tmp = iter.last().unwrap().0;
                Err(tmp..=tmp)
            }
        }
        Distribution {r#"\.\."#}
        OpenBracket {r#"\{"#}
        CloseBracket {r#"}"#}
        Eq {r#"="#}
        Add {r#"\+"#}
        Sub {r#"-"#}
        Mul {r"\*"}
        Div {r#"/"#}
    }
    enums {
        Item -> AnyBlock, AssignExpr, Literal, Ident, Ignore
        AnyBlock -> NamedDistrBlock, DistrBlock, NamedBlock, Block
        AssignExpr -> AssignAnd, Assign
        Literal -> String, Number

        Ignore -> WhiteSpace, NextLine, Tab

        Op -> Add, Sub, Mul, Div

        CacheConstructItem -> Var1, Var2
        CacheConstructHead -> Var3, CommCons1
        CacheToken -> CommCons1, Ident
        CacheEnum -> Var5, Op
        CacheConstructWalkthroug -> Var6, Var7, Var8
    }
    constructs {
        NamedDistrBlock -> NamedBlock (Ignore), Distribution [1]
        DistrBlock -> Ident (Ignore), Distribution [1]
        NamedBlock -> Ident (Ignore), Block [1]
        Block -> OpenBracket, Items, CloseBracket [2]

        AssignAnd -> Ident (Ignore), OpEq (Ignore), Literal [2]
        Assign -> Ident (Ignore), Eq (Ignore), Literal [2]
        OpEq -> Op, Eq [1]

        Tmp -> Ident, Eq [1]

        CommCons1 -> Ident, Distribution [1]

        Var1 -> Ident, Ignore, Eq, Number, WhiteSpace, NamedDistrBlock, Item, Ignore, AssignAnd, WhiteSpace, AssignExpr, Literal, Sub, OpEq, Op, Number [15]
        Var2 -> Ident, Ignore, Eq, Number, WhiteSpace, NamedDistrBlock, Item, Ignore, AssignAnd, WhiteSpace, AssignExpr, Literal, Sub, OpEq, Op [14]

        Var3 -> CommCons1, Ident [1]

        Var5 -> Op, Ident [1]

        Var6 -> Ident, CloseBracket, Distribution [2]
        Var7 -> Ident, CloseBracket, OpenBracket, CloseBracket [3]
        Var8 -> Ident, CloseBracket, OpenBracket, WhiteSpace [3]
    }
);

impl<'a> From<&'a str> for ParseArgs {
    fn from(value: &'a str) -> Self {
        Self::new(&value)
    }
}

#[derive(Debug, Clone)]
pub struct Items(Vec<Item>);
impl Slicable for Items {
    fn get_slice(&self) -> Slice {
        self.0.last().unwrap().get_slice()
    }
}

impl Items {
    pub fn recog(arg: &mut ParseArgs, l: usize) -> Self {
        let mut vec = vec![];
        loop {
            if arg.code.cursor == arg.code.source.len() {
                break;
            }
            match Item::recog(arg, l) {
                Ok(v) => {
                    // не влияет на алгоритм, но очищает ненужную память, ускоряет поиск, в списке
                    arg.c_a_d.borrow_mut().cache.pass.clear();
                    vec.push(v);
                }
                Err(e) => {
                    if arg.code.get_current().1 == '}' {
                        break;
                    } else {
                        arg.code.cursor += 1;
                        println!("ERROR {e:?}");
                        arg.c_a_d.borrow_mut().errors.push(e);
                        continue;
                    }
                }
            };
        }
        Self(vec)
    }
}

mod cache {
    use super::*;
    mod pass {
        use super::*;
        mod construct {
            use super::*;
            /// кеширование головы конструкции. DistructBlock должен парсится 1 раз
            #[test]
            fn head() {
                let mut t = "main..".into();
                dbg!(CacheConstructHead::recog(&mut t, 0));
            }

            /// кеширование элементов конструкции. элемнты Var2 должны распознаться только 1 раз до Var2
            #[test]
            fn item() {
                let mut t = "maijn\t=2323 main { }..sdf\nr+=2 t/=4\"sdfsf sdf\"-+=+".into();
                // let mut t = "mai { }..".into();
                dbg!(CacheConstructItem::recog(&mut t, 0));
            }

            /// После Var7 `goods` от Var6 должны расширятся, а не создаватбся новый список
            #[test]
            fn list_walkthrough() {
                let mut t = "main}{ ".into();
                dbg!(CacheConstructWalkthroug::recog(&mut t, 0));
            }

            mod with {
                use super::*;
                /// Ident должен распознатся 1 раз, как токен без конструкции но часть вариации
                #[test]
                fn token() {
                    let mut t = r#"sdfsdf1."#.into();
                    // 0 -> NamedBlock -> Ident Block -> Fail -> memeory (pos, item)
                    // 0 -> Ident -> (x from memeory)
                    dbg!(CacheToken::recog(&mut t, 0));
                    dbg!(&t.c_a_d.borrow().cache);
                }

                #[test]
                fn enum_() {
                    let mut t = "+ ".into();
                    dbg!(CacheEnum::recog(&mut t, 0));
                }
            }
        }
    }
    mod fail {
        use super::*;

        #[test]
        fn token() {
            let mut t = "2sdfsfd".into();
            dbg!(CacheToken::recog(&mut t, 0));
        }

        #[test]
        fn enum_() {
            let mut t = "22".into();
            dbg!(CacheEnum::recog(&mut t, 0));
        }
    }
}

mod errors {
    use super::*;

    #[test]
    fn items() {
        let mut t = r#"  "sdfsf "#.into();
        dbg!([Construct::WhiteSpace, Construct::String].recog(&mut t, 0));
    }
}

#[test]
fn code() {
    dbg!(Items::recog(
        &mut r#"
            afsg__223
            afsg___

            main {
                a = 2
                a += 200
                a -= 200
                a *= 200
                {
                    {
                        {
                            t
                        }
                    }
                }
            }

            main..

            result = 502
            "#
        .into(),
        0
    ));
}

impl<'a> IntoIterator for &'a Code {
    type Item = &'a (usize, char);
    type IntoIter = std::slice::Iter<'a, (usize, char)>;

    fn into_iter(self) -> Self::IntoIter {
        self.source[self.cursor..].iter()
    }
}
