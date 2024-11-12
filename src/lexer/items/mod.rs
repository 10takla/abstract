pub mod item;
pub mod shared;

use super::{Code, Parse, Slicable};
use item::{
    assign_expr::{assign::Assign, literal::LiteralType, AssignExpr, AssignExprType},
    ident::Ident,
    Item,
};
use std::fmt::Debug;
use std_reset::prelude::Deref;

#[derive(PartialEq, Debug, Deref, Hash, Eq, Clone)]
pub struct Items(pub Vec<Item>);

impl Parse for Items {
    fn parse(code: &Code) -> Option<Self> {
        let mut items = Vec::new();

        let code = &mut code.clone();
        while let Some(item) = Item::parse_and_consume(code) {
            items.push(item);
        }

        Some(Self(items))
    }
}

impl Slicable for Items {
    fn get_start(&self) -> usize {
        self.first().unwrap().get_start()
    }
    fn get_end(&self) -> usize {
        self.last().unwrap().get_end()
    }
}

impl std::fmt::Display for Items {
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
    let check = |a, b: fn(&Code) -> Vec<Item>| {
        let code = &mut Code::new(a);
        assert_eq!(Items::parse(code), Some(Items(b(code))));
    };

    check(" abc = \"abc\"", |code| {
        vec![Item::AssignExpr(AssignExpr {
            type_: AssignExprType::Assign,
            val: Assign::new(1..=3, (LiteralType::String, 7..=11), code),
        })]
    });
    check(" abc = \"abc\" abc", |code| {
        vec![
            Item::AssignExpr(AssignExpr {
                type_: AssignExprType::Assign,
                val: Assign::new(1..=3, (LiteralType::String, 7..=11), code),
            }),
            Item::Ident(Ident::new(13..=15, code)),
        ]
    });

    check("   ", |_| vec![]);

    check(" abc abc ", |code| {
        vec![
            Item::Ident(Ident::new(1..=3, code)),
            Item::Ident(Ident::new(5..=7, code)),
        ]
    });
}
