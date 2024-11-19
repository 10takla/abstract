pub mod assign;
pub mod assign_and;
pub mod literal;

use crate::{
    lexer::{
        check, check_diag, check_none,
        items::{Code, Slicable},
        DiagParse, Diags,
    },
    parse_variants,
};
use assign::{Assign, AssignDiag};
use assign_and::{AssignAnd, AssignAndDiag, AssignAndType};
use core::slice;
use literal::LiteralType;
use macros::Slicable;
use std::{fmt::Display, ops::RangeInclusive};

#[derive(PartialEq, Debug, Clone, Hash, Eq, Slicable)]
pub struct AssignExpr<'s> {
    pub type_: AssignExprType,
    #[slice]
    pub val: Assign<'s>,
}

#[derive(PartialEq, Debug, Clone, Hash, Eq)]
pub enum AssignExprType {
    Assign,
    AssignAnd(AssignAndType),
}

#[derive(PartialEq, Debug)]
pub enum AssignExprDiag {
    Assign(AssignDiag),
    AssignAnd(AssignAndDiag),
}

impl<'s> DiagParse<'s> for AssignExpr<'s> {
    type Diag = AssignExprDiag;

    fn parse(code: &Code<'s>, diags: &mut Diags<Self::Diag>) -> Option<Self> {
        parse_variants!(
            diag diags
            Assign::diag(code).map(|val| Self {
                type_: AssignExprType::Assign,
                val,
            }),
            diag: AssignExprDiag::Assign;
            AssignAnd::diag(code).map(|v| Self {
                type_: AssignExprType::AssignAnd(v.type_),
                val: v.val,
            }),
            diag: AssignExprDiag::AssignAnd
        )
    }
}

impl Display for AssignExpr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self.type_ {
                AssignExprType::Assign => self.val.clone().to_string(),
                AssignExprType::AssignAnd(type_) => {
                    AssignAnd {
                        type_: type_.clone(),
                        val: self.val.clone(),
                    }
                    .to_string()
                }
            }
        )
    }
}

#[test]
fn parse() {
    let check = |source,
                 f: (
        AssignExprType,
        (RangeInclusive<usize>, (LiteralType, RangeInclusive<usize>)),
    )| {
        check(source, |code| AssignExpr {
            type_: f.0,
            val: Assign::new(f.1 .0, (f.1 .1 .0, f.1 .1 .1), code),
        });
    };

    check(
        " abc=6",
        (
            AssignExprType::Assign,
            (1..=3, (LiteralType::Number, 5..=5)),
        ),
    );
    check(
        " abc = 6",
        (
            AssignExprType::Assign,
            (1..=3, (LiteralType::Number, 7..=7)),
        ),
    );
    check(
        " abc = \"root\"",
        (
            AssignExprType::Assign,
            (1..=3, (LiteralType::String, 7..=12)),
        ),
    );
    check(
        " abc += 6",
        (
            AssignExprType::AssignAnd(AssignAndType::Add),
            (1..=3, (LiteralType::Number, 8..=8)),
        ),
    );

    // errors
    check_none::<AssignExpr>("");
    check_none::<AssignExpr>(" ");
}

#[test]
fn diag() {
    check_diag::<AssignExprDiag, AssignExpr>(
        " a - 2",
        vec![
            (3, AssignExprDiag::Assign(AssignDiag::ExpectEqual)),
            (
                4,
                AssignExprDiag::AssignAnd(AssignAndDiag::Assign(AssignDiag::ExpectEqual)),
            ),
        ],
    );
    check_diag::<AssignExprDiag, AssignExpr>(
        " a ( 2",
        vec![
            (3, AssignExprDiag::Assign(AssignDiag::ExpectEqual)),
            (3, AssignExprDiag::AssignAnd(AssignAndDiag::ExpectOperator)),
        ],
    );
}
