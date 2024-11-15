pub mod item;
pub mod shared;

use super::{Code, Parse, Slicable};
use crate::lexer::check;
use item::{
    assign_expr::{assign::Assign, literal::LiteralType, AssignExpr, AssignExprType},
    ident::Ident,
    Item,
};
use std::fmt::Debug;
use std_reset::prelude::Deref;

#[derive(PartialEq, Debug, Deref, Hash, Eq, Clone)]
pub struct Items<'s>(pub Vec<Item<'s>>);

impl<'s> Parse<'s> for Items<'s> {
    fn parse(code: &Code<'s>) -> Option<Self> {
        let mut items = Vec::new();

        let code = &mut code.clone();

        loop {
            if let Some(item) = Item::parse_and_consume(code) {
                items.push(item);
            } else {
                if code.cursor >= code.len() - 1 {
                    break;
                }
                code.offset(1);
            }
            if code.cursor >= code.len() - 1 {
                break;
            }
        }

        Some(Self(items))
    }
}

impl Slicable for Items<'_> {
    fn get_start(&self) -> usize {
        self.first().unwrap().get_start()
    }
    fn get_end(&self) -> usize {
        self.last().unwrap().get_end()
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
fn parse_elements() {
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
fn dota2() {
    dbg!(Items::parse(&Code::new("ddsdfd sdf sdf; sdf sdf s")));
}
