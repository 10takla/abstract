pub mod item;
pub mod shared;

use super::{check_diag, Code, Diag, DiagParse, Diags, Parse, Recognized, Slicable};
use crate::{lexer::check, Diagn};
use item::{
    assign_expr::{assign::Assign, literal::LiteralType, AssignExpr, AssignExprType},
    ident::Ident,
    Item, ItemDiag,
};
use std::{fmt::Debug, ops::RangeInclusive};
use std_reset::prelude::Deref;

#[derive(PartialEq, Debug, Deref, Hash, Eq, Clone)]
pub struct Items<'s>(pub Vec<Item<'s>>);

impl<'s> Items<'s> {
    pub fn analyz(source: &'s str) -> (Self, Vec<Diags<ItemDiag>>) {
        let mut diags = Default::default();
        (
            Items::parse(&source.into(), &mut diags, &mut Default::default()).unwrap(),
            diags,
        )
    }
}

impl<'s> Parse<'s> for Items<'s> {
    type Diag = ItemDiag;
    type Diags = Vec<Diags<Self::Diag>>;
    fn parse(
        code: &Code<'s>,
        diags: &mut Self::Diags,
        recognized: &mut Recognized<'s>,
    ) -> Option<Self> {
        let mut items = Vec::new();

        let code = &mut code.clone();

        loop {
            match Item::diag_and_consume(code, recognized) {
                Ok(item) => {
                    items.push(item);
                }
                Err(d) => {
                    if code.cursor >= code.len() - 1 {
                        break;
                    }
                    code.offset(1);
                    if !d.is_empty() {
                        diags.push(d);
                    }
                }
            }
            if code.cursor >= code.len() - 1 {
                break;
            }
        }

        Some(Self(items))
    }
}

impl Slicable for Items<'_> {
    fn get_slice(&self) -> RangeInclusive<usize> {
        RangeInclusive::new(
            self.first().unwrap().get_start(),
            self.last().unwrap().get_end(),
        )
    }
}

impl std::fmt::Display for Items<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[test]
fn parse() {
    fn fast<'s>(source: &'s str, f: fn(&Code<'s>) -> Vec<Item<'s>>) {
        check(source, |code| Items(f(code)));
    }

    fast(" abc = \"abc\"", |code| {
        vec![Item::AssignExpr(AssignExpr {
            type_: AssignExprType::Assign,
            val: Assign::new(1..=3, (LiteralType::String, 7..=11), code),
        })]
    });
    fast(" abc = \"abc\" abc", |code| {
        vec![
            Item::AssignExpr(AssignExpr {
                type_: AssignExprType::Assign,
                val: Assign::new(1..=3, (LiteralType::String, 7..=11), code),
            }),
            Item::Ident(Ident::new(13..=15, code)),
        ]
    });

    fast("   ", |_| vec![]);

    fast(" aыы aыы", |code| {
        vec![
            Item::Ident(Ident::new(1..=3, code)),
            Item::Ident(Ident::new(5..=7, code)),
        ]
    });

    fast(" aыы;aыы", |code| {
        vec![
            Item::Ident(Ident::new(1..=3, code)),
            Item::Ident(Ident::new(5..=7, code)),
        ]
    });
}

#[test]
fn diag() {
    let mut diags = Default::default();
    let code = &Code::new("m = i");
    Item::parse(code, &mut diags, &mut Default::default());
    for diag in &diags.errors {
        // println!("{}", diag.expect(code, diag));
    }
    
    dbg!(diags);
}
