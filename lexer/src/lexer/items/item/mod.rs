pub mod assign_expr;
pub mod block;
pub mod ident;

use crate::lexer::{check, check_none, Parse, Slicable};
use assign_expr::{assign::Assign, literal::{Literal, LiteralType}, AssignExpr, AssignExprType};
use block::Block;
use ident::Ident;
use macros::Parse;

#[derive(Debug, PartialEq, Parse, Hash, Eq, Clone)]
pub enum Item<'s> {
    Block(Block<'s>),
    AssignExpr(AssignExpr<'s>),
    Ident(Ident<'s>),
    Literal(Literal<'s>),
}

#[test]
fn parse_item() {
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
