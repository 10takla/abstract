use crate::{
    lexer::{
        check, check_diag, check_none, items::shared::whitespaces::Whitespaces, Code, DiagParse,
        Diags, Slice,
    },
    Slicable,
};
use macros::{Diagn, Slicable};
use std::fmt::Display;

#[derive(Debug, PartialEq, Slicable)]
pub struct String<'s>(pub Slice<'s>);

#[derive(PartialEq, Debug, Diagn)]
#[name("String")]
pub enum StringDiag {
    #[diagn_expect("начинаться на [\"]")]
    StartsWithQuote,
    #[diagn_expect("заканчиваться на [\"]")]
    EndsWithQuote,
}

impl Display for StringDiag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use StringDiag::*;
        write!(
            f,
            "{}",
            match self {
                StartsWithQuote => "Должно начинатся с \"",
                EndsWithQuote => "Должно заканчиватся на \"",
            }
        )
    }
}

impl<'s> DiagParse<'s> for String<'s> {
    type Diag = StringDiag;

    fn parse(code: &Code<'s>, diags: &mut Diags<Self::Diag>) -> Option<Self> {
        let code = &mut code.clone();

        Whitespaces::parse_and_consume(code, &mut vec![]);
        let mut iter = code.iter();

        let (i, char) = iter.next()?;
        let start = (char == '"').then_some(i).or_else(|| {
            diags.push((i, StringDiag::StartsWithQuote));
            None
        })?;

        for (i, char) in iter.clone() {
            if char == '"' {
                return Some(Self(Slice::new(start..=i, code)));
            }
        }
        diags.push((iter.last().unwrap().0, StringDiag::EndsWithQuote));
        None
    }
}

#[test]
fn parse() {
    let check = |source, range| {
        check(source, |code| String(Slice::new(range, code)));
    };

    check(r#""abc""#, 0..=4);
    check(r#""abc"  "#, 0..=4);
    check(r#"  "abc"  "#, 2..=6);
    check(r#"  " ab s3fsf d2_c "  "#, 2..=18);
    check(r#""""#, 0..=1);
    check(r#" " " "#, 1..=3);

    // errors
    check_none::<String>(" 2sdf ");
    check_none::<String>(" \"2sdf ");
    check_none::<String>(" \" ");
}

#[test]
fn diag() {
    check_diag::<StringDiag, String>("  43c", vec![(2, StringDiag::StartsWithQuote)]);
    check_diag::<StringDiag, String>("  \"4c", vec![(4, StringDiag::EndsWithQuote)]);
}
