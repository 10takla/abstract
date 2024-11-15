use crate::lexer::{items::shared::whitespaces::Whitespaces, Code, Parse, Slicable, Slice};
use std::{fmt::Display, ops::RangeInclusive};
use std_reset::prelude::Deref;

#[derive(PartialEq, Debug, Clone, Deref, Eq, Hash)]
pub struct Ident<'s>(pub Slice<'s>);

impl<'s> Ident<'s> {
    pub fn new(range: RangeInclusive<usize>, code: &Code<'s>) -> Self {
        Self(Slice::new(range, code))
    }
}

impl Display for Ident<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ident({})", self.0)
    }
}

impl<'s> Parse<'s> for Ident<'s> {
    fn parse(code: &Code<'s>) -> Option<Self> {
        let code = &mut code.clone();

        let start_rule = |char: char| char.is_alphabetic() || char == '_';

        Whitespaces::parse_and_consume(code);

        let mut iter = code.iter();
        let (i, char) = iter.next()?;
        let start = start_rule(char).then_some(i)?;

        let end = if start == code.len() - 1 {
            start
        } else {
            iter.find_map(|(i, char)| {
                if start_rule(char) || char.is_digit(10) {
                    (i == code.len() - 1).then_some(i)
                } else {
                    Some(i - 1)
                }
            })?
        };

        Some(Self::new(start..=end, code))
    }
}

impl Slicable for Ident<'_> {
    fn get_start(&self) -> usize {
        self.0.get_start()
    }
    fn get_end(&self) -> usize {
        self.0.get_end()
    }
}

#[test]
fn parse_ident() {
    let check = |a, b| {
        let code = &mut Code::new(a);
        assert_eq!(Ident::parse(code), Some(Ident::new(b, code)));
    };
    let check_none = |a| {
        assert_eq!(Ident::parse(&mut Code::new(a)), None);
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

    // errors
    check_none("  2sdf ");
    check_none("  ");
}
