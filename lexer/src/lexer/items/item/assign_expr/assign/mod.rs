pub mod left_right;

use super::literal::{Literal, LiteralType};
use crate::{
    lexer::{
        check, check_none,
        items::{item::ident::Ident, Code},
        Parse, Slicable,
    },
    Slice,
};
use left_right::LeftRight;
use std::{fmt::Display, ops::RangeInclusive};
use std_reset::prelude::Deref;

#[derive(PartialEq, Debug, Clone, Deref, Hash, Eq)]
pub struct Assign<'s>(pub LeftRight<'s, Ident<'s>, Literal<'s>>);

impl<'s> Assign<'s> {
    pub fn new(
        ident_slice: RangeInclusive<usize>,
        (literal_type, literal_slice): (LiteralType, RangeInclusive<usize>),
        code: &Code<'s>,
    ) -> Self {
        Self(LeftRight {
            left: Ident::new(ident_slice, code),
            right: Literal::new(literal_type, literal_slice, code),
            _marker: Default::default(),
        })
    }
}

impl<'s> Parse<'s> for Assign<'s> {
    fn parse(code: &Code<'s>) -> Option<Self> {
        LeftRight::parse(code, |code| {
            let (i, char) = code.iter().next()?;
            (char == '=').then_some(i)
        })
        .map(Self)
    }
}

impl Slicable for Assign<'_> {
    fn get_start(&self) -> usize {
        self.left.get_start()
    }
    fn get_end(&self) -> usize {
        self.right.get_end()
    }
}

impl Display for Assign<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AssignAnd({} = {})", self.left, self.right)
    }
}

#[test]
fn parse_assign() {
    let check = |source, v: (RangeInclusive<usize>, (LiteralType, RangeInclusive<usize>))| {
        check(source, |code| {
            Assign(LeftRight {
                left: Ident::new(v.0, code),
                right: Literal {
                    type_: v.1 .0,
                    slice: Slice::new(v.1 .1, code),
                },
                _marker: Default::default(),
            })
        });
    };

    check(" abc = 6", (1..=3, (LiteralType::Number, 7..=7)));
    check("abc=6", (0..=2, (LiteralType::Number, 4..=4)));
    check(" abc=6 ", (1..=3, (LiteralType::Number, 5..=5)));
    check(" abc = \"root\"", (1..=3, (LiteralType::String, 7..=12)));

    check_none::<Assign>("abc =");
    check_none::<Assign>("abc = ");
    check_none::<Assign>("abc=");
}
