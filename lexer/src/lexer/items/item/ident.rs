use crate::lexer::{
    check, check_diag, check_none, items::shared::whitespaces::Whitespaces, Code, Diag, DiagParse,
    Diags, Slicable, Slice,
};
use colored::Colorize;
use macros::{Diagn, Slicable};
use std::{fmt::Display, ops::RangeInclusive};
use std_reset::prelude::Deref;

#[derive(PartialEq, Debug, Clone, Deref, Eq, Hash, Slicable)]
pub struct Ident<'s>(pub Slice<'s>);

impl<'s> Ident<'s> {
    pub fn new(range: RangeInclusive<usize>, code: &Code<'s>) -> Self {
        Self(Slice::new(range, code))
    }
}

#[derive(PartialEq, Debug, Clone, Eq, Hash, Diagn)]
#[name("Ident")]
pub enum IdentDiag {
    #[diagn_expect("начинаться [a-z|A-Z]")]
    StartsWithNotNumber,
}

impl<'s> DiagParse<'s> for Ident<'s> {
    type Diag = IdentDiag;

    fn parse(code: &Code<'s>, diags: &mut Diags<Self::Diag>) -> Option<Self> {
        let code = &mut code.clone();

        let start_rule = |char: char| char.is_alphabetic() || char == '_';

        Whitespaces::parse_and_consume(code, &mut vec![]);

        let mut iter = code.iter();
        let (i, char) = iter.next()?;
        let start = start_rule(char).then_some(i).or_else(|| {
            diags.push((i, IdentDiag::StartsWithNotNumber));
            None
        })?;

        let end = if start == code.len() - 1 {
            start
        } else {
            iter.find_map(|(i, char)| {
                if start_rule(char) || char.is_digit(10) {
                    None
                } else {
                    Some(i - 1)
                }
            })
            .unwrap_or(code.len() - 1)
        };

        Some(Self::new(start..=end, code))
    }
}

impl Display for Ident<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ident({})", self.0)
    }
}

#[test]
fn parse() {
    let check = |source, range| {
        check(source, move |code| Ident::new(range, code));
    };

    check("abc", 0..=2);
    check("  abc", 2..=4);
    check("  фbc  ", 2..=4);
    check("abc123", 0..=5);
    check(" abc__123 ", 1..=8);
    check(" _123 ", 1..=4);
    check(" abc_=s23 ", 1..=4);

    check("a", 0..=0);
    check(" a ", 1..=1);
    check(" _ ", 1..=1);

    check(" фc", 1..=2);
    check(" ффф ", 1..=3);
    check(" dъя", 1..=3);
    check(" d;", 1..=1);

    // errors
    check_none::<Ident>("  2sdf ");
    check_none::<Ident>("  ");
}

#[test]
fn diag() {
    check_diag::<IdentDiag, Ident>(" 2sdf", vec![(1, IdentDiag::StartsWithNotNumber)]);
    check_diag::<IdentDiag, Ident>("  ", vec![]);
}
