pub mod assign_expr;
pub mod block;
pub mod ident;

use crate::lexer::{check, check_none, Parse, Slicable};
use assign_expr::{assign::Assign, literal::LiteralType, AssignExpr, AssignExprType};
use block::Block;
use ident::Ident;
use macros::Parse;

#[derive(Debug, PartialEq, Parse)]
pub enum Item {
    Block(Block),
    AssignExpr(AssignExpr),
    Ident(Ident),
}

#[test]
fn parse_element() {
    check(" abc = \"abc\"", |code| {
        Item::AssignExpr(AssignExpr {
            type_: AssignExprType::Assign,
            val: Assign::new([1, 3], (LiteralType::String, [7, 11]), code),
        })
    });
    check(" abc = \"a", |code| Item::Ident(Ident::new([1, 3], code)));

    // errors
    check_none::<Item>("  ");
}
