pub mod assign_expr;
pub mod block;
pub mod ident;

use super::shared::distribution::{self, DistributionDiag};
use crate::{
    items::Items,
    lexer::{check, check_diag, check_none, DiagParse, Diags, SelectionParse, Slicable},
    Code, Diagn, Parse,
};
use assign_expr::{
    assign::{
        left_right::{LeftRight, LeftRightDiag},
        Assign, AssignDiag,
    },
    assign_and::AssignAndDiag,
    literal::{number::NumberDiag, string::StringDiag, Literal, LiteralDiag, LiteralType},
    AssignExpr, AssignExprDiag, AssignExprType,
};
use block::{
    distruct::{
        call::{self, CallBlockDistructDiag},
        init::InitBlockDistructDiag,
        BlockDistructDiag,
    },
    init::{
        named::NamedBlockDiag,
        unnamed::{self, UnnamedBlockDiag},
        InitBlockDiag,
    },
    Block, BlockDiag,
};
use colored::Colorize;
use ident::{Ident, IdentDiag};
use macros::Parse;
use std::{
    fmt::{format, Debug, Display},
    hash::DefaultHasher,
};
use std_reset::prelude::Display;

#[derive(Debug, PartialEq, Parse, Hash, Eq, Clone)]
#[diag(ItemDiag)]
pub enum Item<'s> {
    Block(Block<'s>),
    AssignExpr(AssignExpr<'s>),
    Ident(Ident<'s>),
    Literal(Literal<'s>),
}

#[derive(PartialEq, Debug, Clone)]
pub enum ItemDiag {
    Block(BlockDiag),
    AssignExpr(AssignExprDiag),
    Ident(IdentDiag),
    Literal(LiteralDiag),
}

#[test]
fn parse() {
    check(" abc = \"abc\"", |code| {
        Item::AssignExpr(AssignExpr {
            type_: AssignExprType::Assign,
            val: Assign::new(1..=3, (LiteralType::String, 7..=11), code),
        })
    });
    check(" abc = \"a", |code| Item::Ident(Ident::new(1..=3, code)));

    // errors
    check_none::<Item>("  ");
}

impl Diagn for AssignAndDiag {
    const NAME: &'static str = "Assign";
    fn expect(&self, code: &Code, pos: usize) -> impl Display {
        match self {
            Self::Assign(assign) => assign.expect(code, pos).to_string(),
            Self::ExpectOperator => "оператор [+|-|*|/]".to_string(),
        }
    }
}

impl Diagn for AssignDiag {
    const NAME: &'static str = "Assign";
    fn expect(&self, code: &Code, pos: usize) -> impl Display {
        match self {
            Self::ExpectEqual => "[=]".to_string(),
            Self::LeftRight(lr) => match lr {
                LeftRightDiag::Left(ident) => ident.expect(code, pos).to_string(),
                LeftRightDiag::Right(literal) => literal.expect(code, pos).to_string(),
            },
        }
    }
}

impl Diagn for LiteralDiag {
    const NAME: &'static str = "Item";
    fn expect(&self, code: &Code, pos: usize) -> impl Display {
        match self {
            Self::Number(n) => n.expect(code, pos).to_string(),
            Self::String(s) => s.expect(code, pos).to_string(),
        }
    }
}

impl Diagn for UnnamedBlockDiag {
    const NAME: &'static str = "Block";
    fn expect(&self, code: &Code, pos: usize) -> impl Display {
        match self {
            Self::StartsOpenBracket => "начинаться на [{]",
            Self::EndsOpenBracket => "заканчиваться на [}]",
            _ => Default::default(),
        }
    }
}
impl Diagn for DistributionDiag {
    const NAME: &'static str = "Distribution";
    fn expect(&self, code: &Code, pos: usize) -> impl Display {
        "[..]"
    }
}
impl Diagn for NamedBlockDiag {
    const NAME: &'static str = "NamedBlock";
    fn expect(&self, code: &Code, pos: usize) -> impl Display {
        match self {
            Self::Name(name) => name.expect(code, pos).to_string(),
            Self::UnnamedBlock(unnamed) => unnamed.expect(code, pos).to_string(),
        }
    }
}

impl Diagn for ItemDiag {
    const NAME: &'static str = "Item";
    fn expect(&self, code: &Code, pos: usize) -> impl Display {
        match self {
            Self::Block(block) => match block {
                BlockDiag::Distruct(distruct) => match distruct {
                    BlockDistructDiag::Call(call) => match call {
                        CallBlockDistructDiag::Name(name) => {
                            name.for_construct(code, pos).to_string()
                        }
                        CallBlockDistructDiag::Distribution(distribution) => {
                            distribution.for_construct(code, pos).to_string()
                        }
                    },
                    BlockDistructDiag::Init(init) => match init {
                        InitBlockDistructDiag::Distribution(distribution) => {
                            distribution.for_construct(code, pos).to_string()
                        }
                        InitBlockDistructDiag::NamedBlock(named) => {
                            named.for_construct(code, pos).to_string()
                        }
                    },
                },
                BlockDiag::Init(init) => match init {
                    InitBlockDiag::Unnamed(unnamed) => unnamed.for_construct(code, pos).to_string(),
                    InitBlockDiag::Named(named) => named.for_construct(code, pos).to_string(),
                },
            },
            Self::AssignExpr(assign_expr) => match assign_expr {
                AssignExprDiag::Assign(assign) => assign.for_construct(code, pos).to_string(),
                AssignExprDiag::AssignAnd(assign_and) => {
                    assign_and.for_construct(code, pos).to_string()
                }
            },
            Self::Literal(literal_diag) => literal_diag.for_construct(code, pos).to_string(),
            Self::Ident(ident_diag) => ident_diag.for_construct(code, pos).to_string(),
        }
    }
}

#[test]
fn diag() {
    let code = &Code::new("  43c");
    let diag = Item::diag(code, &mut Default::default()).unwrap_err();
    diag.iter().for_each(|v| {
        println!("{:?} {}", &v, v.expect(code, diag.pos.unwrap()));
    });

    // check_diag::<ItemDiag, Item>(
    //     "  43c",
    //     vec![
    //         (
    //             2,
    //             ItemDiag::Literal(LiteralDiag::String(StringDiag::StartsWithQuote)),
    //         ),
    //         (
    //             4,
    //             ItemDiag::Literal(LiteralDiag::Number(NumberDiag::MustBeNumber)),
    //         ),
    //     ],
    // );
    // check_diag::<ItemDiag, Item>(
    //     " \"43c",
    //     vec![
    //         (
    //             2,
    //             ItemDiag::AssignExpr(AssignExprDiag::Assign(AssignDiag::LeftRight(
    //                 LeftRightDiag::Left(IdentDiag::StartsWithNotNumber),
    //             ))),
    //         ),
    //         (2, ItemDiag::Ident(IdentDiag::StartsWithNotNumber)),
    //     ],
    // );
}

#[test]
#[ignore]
fn recognize() {
    let mut t = Default::default();
    Item::parse(&"a {".into(), &mut t, &mut Default::default());
    dbg!(t);
}

trait Tmp {
    fn tmp<'a>(&'a self, memo: &mut Vec<&'a dyn Terminal>) {}
}

impl Tmp for ItemDiag {
    fn tmp<'a>(&'a self, memo: &mut Vec<&'a dyn Terminal>) {
        match self {
            Self::Block(v) => v.tmp(memo),
            Self::AssignExpr(v) => v.tmp(memo),
            Self::Literal(v) => v.tmp(memo),
            Self::Ident(v) => v.tmp(memo),
        }
    }
}

impl Tmp for BlockDiag {
    fn tmp<'a>(&'a self, memo: &mut Vec<&'a dyn Terminal>) {
        match self {
            Self::Distruct(v) => v.tmp(memo),
            Self::Init(v) => v.tmp(memo),
        }
    }
}

impl Tmp for BlockDistructDiag {
    fn tmp<'a>(&'a self, memo: &mut Vec<&'a dyn Terminal>) {
        match self {
            Self::Init(v) => v.tmp(memo),
            Self::Call(v) => v.tmp(memo),
        }
    }
}
impl Tmp for InitBlockDistructDiag {
    fn tmp<'a>(&'a self, memo: &mut Vec<&'a dyn Terminal>) {
        match self {
            Self::Distribution(v) => v.tmp(memo),
            Self::NamedBlock(v) => v.tmp(memo),
        }
    }
}
impl Tmp for CallBlockDistructDiag {
    fn tmp<'a>(&'a self, memo: &mut Vec<&'a dyn Terminal>) {
        match self {
            Self::Name(v) => v.tmp(memo),
            Self::Distribution(v) => v.tmp(memo),
        }
    }
}

impl Tmp for InitBlockDiag {
    fn tmp<'a>(&'a self, memo: &mut Vec<&'a dyn Terminal>) {
        match self {
            Self::Named(v) => v.tmp(memo),
            Self::Unnamed(v) => v.tmp(memo),
        }
    }
}
impl Tmp for NamedBlockDiag {
    fn tmp<'a>(&'a self, memo: &mut Vec<&'a dyn Terminal>) {
        match self {
            Self::Name(v) => v.tmp(memo),
            Self::UnnamedBlock(v) => v.tmp(memo),
        }
    }
}
impl Tmp for UnnamedBlockDiag {
    fn tmp<'a>(&'a self, memo: &mut Vec<&'a dyn Terminal>) {
        match self {
            Self::StartsOpenBracket => memo.push(self as &dyn Terminal),
            Self::EndsOpenBracket => memo.push(self as &dyn Terminal),
            _ => Default::default(),
        }
    }
}

impl Tmp for AssignExprDiag {
    fn tmp<'a>(&'a self, memo: &mut Vec<&'a dyn Terminal>) {
        match self {
            Self::Assign(v) => v.tmp(memo),
            Self::AssignAnd(v) => v.tmp(memo),
        }
    }
}

impl Tmp for AssignAndDiag {
    fn tmp<'a>(&'a self, memo: &mut Vec<&'a dyn Terminal>) {
        match self {
            Self::Assign(v) => v.tmp(memo),
            Self::ExpectOperator => memo.push(self as &dyn Terminal),
        }
    }
}

impl Tmp for AssignDiag {
    fn tmp<'a>(&'a self, memo: &mut Vec<&'a dyn Terminal>) {
        match self {
            Self::LeftRight(v) => v.tmp(memo),
            Self::ExpectEqual => memo.push(self as &dyn Terminal),
        }
    }
}
impl<L: Tmp, R: Tmp> Tmp for LeftRightDiag<L, R> {
    fn tmp<'a>(&'a self, memo: &mut Vec<&'a dyn Terminal>) {
        match self {
            Self::Left(v) => v.tmp(memo),
            Self::Right(v) => v.tmp(memo),
        }
    }
}

impl Tmp for LiteralDiag {
    fn tmp<'a>(&'a self, memo: &mut Vec<&'a dyn Terminal>) {
        match self {
            Self::Number(v) => v.tmp(memo),
            Self::String(v) => v.tmp(memo),
        }
    }
}

impl Tmp for NumberDiag {
    fn tmp<'a>(&'a self, memo: &mut Vec<&'a dyn Terminal>) {
        memo.push(self as &dyn Terminal);
    }
}
impl Tmp for StringDiag {
    fn tmp<'a>(&'a self, memo: &mut Vec<&'a dyn Terminal>) {
        memo.push(self as &dyn Terminal);
    }
}

impl Tmp for IdentDiag {
    fn tmp<'a>(&'a self, memo: &mut Vec<&'a dyn Terminal>) {
        memo.push(self as &dyn Terminal);
    }
}
impl Tmp for DistributionDiag {
    fn tmp<'a>(&'a self, memo: &mut Vec<&'a dyn Terminal>) {
        memo.push(self as &dyn Terminal);
    }
}

trait Terminal: Debug {
    fn get_info(&self) -> String;
}
impl Terminal for IdentDiag {
    fn get_info(&self) -> String {
        "IdentDiag".into()
    }
}
impl Terminal for NumberDiag {
    fn get_info(&self) -> String {
        "NumberDiag".into()
    }
}
impl Terminal for StringDiag {
    fn get_info(&self) -> String {
        "StringDiag".into()
    }
}
impl Terminal for DistributionDiag {
    fn get_info(&self) -> String {
        "DistributionDiag".into()
    }
}

impl Terminal for UnnamedBlockDiag {
    fn get_info(&self) -> String {
        "UnnamedBlockDiag".into()
    }
}
impl Terminal for AssignAndDiag {
    fn get_info(&self) -> String {
        "AssignAndDiag".into()
    }
}
impl Terminal for AssignDiag {
    fn get_info(&self) -> String {
        "AssignDiag".into()
    }
}

#[test]
fn dota() {
    let diags = Item::diag(&Code::new(" 43c"), &mut Default::default()).unwrap_err();
    let mut memo = vec![];
    for diag in diags.iter() {
        diag.tmp(&mut memo);
    }
    dbg!(memo
        .into_iter()
        .map(|v| { format!("{} {:?}", v.get_info(), v) })
        .collect::<Vec<_>>());
}
