use super::{
    assign::{
        left_right::{LeftRight, LeftRightDiag},
        parse_equal, Assign, AssignDiag,
    },
    literal::{LiteralDiag, LiteralType},
};
use crate::{
    items::item::ident::IdentDiag,
    lexer::{
        check, check_diag, check_none,
        items::{Code, Slicable},
        DiagParse, Diags,
    },
    Parse, Recognized,
};
use macros::Slicable;
use std::{fmt::Display, hash::DefaultHasher, ops::RangeInclusive};

#[derive(PartialEq, Debug, Clone, Slicable)]
pub struct AssignAnd<'s> {
    pub type_: AssignAndType,
    #[slice]
    pub val: Assign<'s>,
}

#[derive(PartialEq, Debug, Clone, Hash, Eq)]
pub enum AssignAndType {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(PartialEq, Debug, Clone)]
pub enum AssignAndDiag {
    Assign(AssignDiag),
    ExpectOperator,
}

impl<'s> Parse<'s> for AssignAnd<'s> {
    type Diag = AssignAndDiag;

    fn parse(
        code: &Code<'s>,
        diags: &mut Diags<Self::Diag>,
        recognized: &mut Recognized<'s>,
    ) -> Option<Self> {
        let mut assign_type = None;
        let mut d = Default::default();
        LeftRight::parse(code, &mut d, recognized, |code| {
            let mut iter = code.iter();

            let (i, char) = iter.next()?;
            match char {
                '+' => assign_type = Some(AssignAndType::Add),
                '-' => assign_type = Some(AssignAndType::Sub),
                '*' => assign_type = Some(AssignAndType::Mul),
                '/' => assign_type = Some(AssignAndType::Div),
                _ => {
                    diags.extend_one((i, AssignAndDiag::ExpectOperator));
                    return None;
                }
            }

            let mut d = Default::default();
            parse_equal(iter, &mut d).or_else(|| {
                diags.extend(d.iter().cloned().map(AssignAndDiag::Assign));
                None
            })
        })
        .map(|lr| Self {
            type_: assign_type.unwrap(),
            val: Assign(lr),
        })
    }
}

impl<'s> DiagParse<'s> for AssignAnd<'s> {}

impl<'s> AssignAnd<'s> {
    pub fn new(
        type_: AssignAndType,
        (slice, (literal_type, literal_slice)): (
            RangeInclusive<usize>,
            (LiteralType, RangeInclusive<usize>),
        ),
        code: &Code<'s>,
    ) -> Self {
        Self {
            type_,
            val: Assign::new(slice, (literal_type, literal_slice), code),
        }
    }
}

impl Display for AssignAnd<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AssignAnd({} {} {})",
            self.val.left, self.type_, self.val.right,
        )
    }
}

impl Display for AssignAndType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AssignAndType::Add => "Add(+=)",
                AssignAndType::Sub => "Sub(-=)",
                AssignAndType::Mul => "Mul(*=)",
                AssignAndType::Div => "Div(/=)",
            },
        )
    }
}

#[test]
fn parse() {
    check(" abc += 6", |code| {
        AssignAnd::new(
            AssignAndType::Add,
            (1..=3, (LiteralType::Number, 8..=8)),
            code,
        )
    });
    check(" abc -= 6", |code| {
        AssignAnd::new(
            AssignAndType::Sub,
            (1..=3, (LiteralType::Number, 8..=8)),
            code,
        )
    });
    check(" abc *= 6", |code| {
        AssignAnd::new(
            AssignAndType::Mul,
            (1..=3, (LiteralType::Number, 8..=8)),
            code,
        )
    });
    check(" abc /= 6", |code| {
        AssignAnd::new(
            AssignAndType::Div,
            (1..=3, (LiteralType::Number, 8..=8)),
            code,
        )
    });

    check("abc+=6", |code| {
        AssignAnd::new(
            AssignAndType::Add,
            (0..=2, (LiteralType::Number, 5..=5)),
            code,
        )
    });

    // error
    check_none::<AssignAnd>("a + ");
    check_none::<AssignAnd>("");
    check_none::<AssignAnd>(" ");
    check_none::<AssignAnd>("   ");
}

#[test]
fn diag() {
    check_diag::<AssignAndDiag, AssignAnd>(
        " a - 2",
        vec![(4, AssignAndDiag::Assign(AssignDiag::ExpectEqual))],
    );
    check_diag::<AssignAndDiag, AssignAnd>(" a ( 2", vec![(3, AssignAndDiag::ExpectOperator)]);
}
