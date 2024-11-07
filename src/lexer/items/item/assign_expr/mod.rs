pub mod assign;
pub mod assign_and;
pub mod literal;

use crate::{
    lexer::{
        items::{Code, Slicable},
        Parse,
    },
    parse_variants,
};
use assign::Assign;
use assign_and::{AssignAnd, AssignAndType};
use literal::LiteralType;
use std::fmt::Display;

#[derive(PartialEq, Debug, Clone, Hash, Eq)]
pub struct AssignExpr {
    pub type_: AssignExprType,
    pub val: Assign,
}

#[derive(PartialEq, Debug, Clone, Hash, Eq)]
pub enum AssignExprType {
    Assign,
    AssignAnd(AssignAndType),
}

impl Parse for AssignExpr {
    fn parse(code: &Code) -> Option<Self> {
        parse_variants!(
            Assign::parse(code).map(|val| Self {
                type_: AssignExprType::Assign,
                val,
            }),
            AssignAnd::parse(code).map(|v| Self {
                type_: AssignExprType::AssignAnd(v.type_),
                val: v.val,
            })
        )
    }
}

impl Slicable for AssignExpr {
    fn get_end(&self) -> usize {
        self.val.get_end()
    }
}

impl Display for AssignExpr {
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
fn parse_assign_expr() {
    let check = |a, b: (AssignExprType, ([usize; 2], (LiteralType, [usize; 2])))| {
        let code = &mut Code::new(a);
        assert_eq!(
            AssignExpr::parse(code),
            Some(AssignExpr {
                type_: b.0,
                val: Assign::new(b.1 .0, (b.1 .1 .0, b.1 .1 .1), code)
            })
        );
    };
    let check_none = |a| {
        assert_eq!(AssignExpr::parse(&mut Code::new(a)), None);
    };

    check(
        " abc=6",
        (
            AssignExprType::Assign,
            ([1, 3], (LiteralType::Number, [5, 5])),
        ),
    );
    check(
        " abc = 6",
        (
            AssignExprType::Assign,
            ([1, 3], (LiteralType::Number, [7, 7])),
        ),
    );
    check(
        " abc = \"root\"",
        (
            AssignExprType::Assign,
            ([1, 3], (LiteralType::String, [7, 12])),
        ),
    );
    check(
        " abc += 6",
        (
            AssignExprType::AssignAnd(AssignAndType::Add),
            ([1, 3], (LiteralType::Number, [8, 8])),
        ),
    );

    // errors
    check_none("");
    check_none(" ");
}
