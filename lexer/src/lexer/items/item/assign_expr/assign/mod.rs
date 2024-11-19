pub mod left_right;

use super::literal::{Literal, LiteralDiag, LiteralType};
use crate::{
    items::item::ident::IdentDiag,
    lexer::{
        check, check_diag, check_none,
        items::{item::ident::Ident, Code},
        DiagParse, Diags, Slicable,
    },
    Slice,
};
use left_right::{LeftRight, LeftRightDiag};
use macros::Slicable;
use std::{fmt::Display, ops::RangeInclusive};
use std_reset::prelude::Deref;

#[derive(PartialEq, Debug, Clone, Deref, Hash, Eq, Slicable)]
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

#[derive(PartialEq, Debug)]
pub enum AssignDiag {
    LeftRight(LeftRightDiag<IdentDiag, LiteralDiag>),
    ExpectEqual,
}

impl<'s> DiagParse<'s> for Assign<'s> {
    type Diag = AssignDiag;

    fn parse(code: &Code<'s>, diags: &mut Diags<Self::Diag>) -> Option<Self> {
        let mut d = vec![];
        LeftRight::parse(code, &mut d, |code| parse_equal(code.iter(), diags))
            .map(Self)
            .or_else(|| {
                diags.extend(d.into_iter().map(|(i, d)| (i, AssignDiag::LeftRight(d))));
                None
            })
    }
}

pub fn parse_equal(
    mut iter: impl Iterator<Item = (usize, char)>,
    diags: &mut Diags<AssignDiag>,
) -> Option<usize> {
    let (i, char) = iter.next()?;
    (char == '=').then_some(i).or_else(|| {
        diags.push((i, AssignDiag::ExpectEqual));
        None
    })
}

impl Display for Assign<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AssignAnd({} = {})", self.left, self.right)
    }
}

#[test]
fn parse() {
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

#[test]
fn diag() {
    check_diag::<AssignDiag, Assign>(" a - 2", vec![(3, AssignDiag::ExpectEqual)]);
}
