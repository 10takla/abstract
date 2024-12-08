use crate::{
    lexer::{
        check, check_diag, check_none, items::shared::whitespaces::Whitespaces, Code, DiagParse,
        Diags, Slice, IGNORE,
    }, Parse, Recognized, Slicable
};
use macros::{Diagn, Slicable};
use std::fmt::Display;

#[derive(PartialEq, Debug, Slicable, Clone)]
pub struct Number<'s>(pub Slice<'s>);

impl<'s> Parse<'s> for Number<'s> {
    type Diag = NumberDiag;

    fn parse(code: &Code<'s>, diags: &mut Diags<Self::Diag>, recognized: &mut Recognized<'s>) -> Option<Self> {
        let code = &mut code.clone();

        Whitespaces::diag_and_consume(code, recognized);
        let mut iter = code.iter();

        let (i, char) = iter.next()?;
        let start = if char.is_digit(10) {
            if i == code.len() - 1 {
                return Some(Self(Slice::new(i..=i, code)));
            }
            i
        } else {
            diags.extend_one((i, NumberDiag::StartsWithNumber));
            return None;
        };

        let end = (|| {
            for (i, char) in iter.clone() {
                if IGNORE.contains(&char) {
                    return Some(i - 1);
                }
                if char.is_digit(10) {
                    if i == code.len() - 1 {
                        return Some(i);
                    }
                    continue;
                }
                diags.extend_one((i, NumberDiag::MustBeNumber));
                return None;
            }
            None
        })()?;

        Some(Self(Slice::new(start..=end, code)))
    }
}

impl<'s> DiagParse<'s> for Number<'s> {}

#[test]
fn parse() {
    let check = |source, v| {
        check(source, |code| Number(Slice::new(v, code)));
    };

    check("2", 0..=0);
    check("2 ", 0..=0);
    check(" 2", 1..=1);
    check("  2  ", 2..=2);
    check("  233", 2..=4);
    check("  443  ", 2..=4);
    check("3434", 0..=3);

    // errors
    check_none::<Number>("");
    check_none::<Number>("  ");
    check_none::<Number>("  2sdf ");
    check_none::<Number>("  abc  ");
    check_none::<Number>("abc123");
}

#[derive(PartialEq, Debug, Diagn, Clone)]
#[name("Name")]
pub enum NumberDiag {
    #[diagn_expect("начинатся на [0-9]")]
    StartsWithNumber,
    #[diagn_expect("[0-9]")]
    MustBeNumber,
}

impl Display for NumberDiag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use NumberDiag::*;
        write!(
            f,
            "{}",
            match self {
                StartsWithNumber => "Должно начинатся с числа",
                MustBeNumber => "Должно быть число",
            }
        )
    }
}

#[test]
fn diag() {
    check_diag::<NumberDiag, Number>("  43c", vec![(4, NumberDiag::MustBeNumber)]);
    check_diag::<NumberDiag, Number>("  4c", vec![(3, NumberDiag::MustBeNumber)]);
    check_diag::<NumberDiag, Number>("  c", vec![(2, NumberDiag::StartsWithNumber)]);
}
