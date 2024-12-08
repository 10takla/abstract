pub mod number;
pub mod string;

use crate::{
    lexer::{check, check_diag, check_none, Code, DiagParse, Diags, Slicable, Slice},
    parse_variants, Parse, Recognized,
};
use macros::{Slicable};
use number::{Number, NumberDiag};
use std::{fmt::Display, ops::RangeInclusive};
use string::{String, StringDiag};

#[derive(PartialEq, Debug, Clone, Hash, Eq, Slicable)]
pub struct Literal<'s> {
    pub type_: LiteralType,
    #[slice]
    pub slice: Slice<'s>,
}

#[derive(PartialEq, Debug, Clone, Hash, Eq)]
pub enum LiteralType {
    Number,
    String,
}

#[derive(PartialEq, Debug, Clone)]
pub enum LiteralDiag {
    Number(NumberDiag),
    String(StringDiag),
}

impl<'s> Parse<'s> for Literal<'s> {
    type Diag = LiteralDiag;

    fn parse(
        code: &Code<'s>,
        diags: &mut Diags<Self::Diag>,
        recognized: &mut Recognized<'s>,
    ) -> Option<Self> {
        parse_variants!(
            diag diags
            Number::diag(code, recognized)
                .map(|v| Self {
                    type_: LiteralType::Number,
                    slice: v.0,
                }),
            diag: Number;
            String::diag(code, recognized)
                .map(|v| Self {
                    type_: LiteralType::String,
                    slice: v.0,
                }),
            diag: String
        )
    }
}

impl<'s> DiagParse<'s> for Literal<'s> {}

impl<'s> Literal<'s> {
    pub fn new(type_: LiteralType, slice: RangeInclusive<usize>, code: &Code<'s>) -> Self {
        Self {
            type_,
            slice: Slice::new(slice, code),
        }
    }
}

impl Display for Literal<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Literal({:?}({}))", self.type_, self.slice)
    }
}

#[test]
fn parse() {
    let check = |source, v: (LiteralType, RangeInclusive<usize>)| {
        check(source, |code| Literal {
            type_: v.0,
            slice: Slice::new(v.1, code),
        });
    };

    check("2", (LiteralType::Number, 0..=0));
    check("2 ", (LiteralType::Number, 0..=0));
    check(" 2", (LiteralType::Number, 1..=1));
    check("  2  ", (LiteralType::Number, 2..=2));
    check("  233", (LiteralType::Number, 2..=4));
    check("  443  ", (LiteralType::Number, 2..=4));
    check("3434", (LiteralType::Number, 0..=3));

    check(r#""abc""#, (LiteralType::String, 0..=4));
    check(r#""abc"  "#, (LiteralType::String, 0..=4));
    check(r#"  "abc"  "#, (LiteralType::String, 2..=6));
    check(r#"  " ab s3fsf d2_c "  "#, (LiteralType::String, 2..=18));

    // errors
    check_none::<Literal>(" 2sdf ");
    check_none::<Literal>(" \"2sdf ");
}

#[test]
fn diag() {
    check_diag::<LiteralDiag, Literal>(
        "  43c",
        vec![
            (4, LiteralDiag::Number(NumberDiag::MustBeNumber)),
            (2, LiteralDiag::String(StringDiag::StartsWithQuote)),
        ],
    );
    check_diag::<LiteralDiag, Literal>(
        " \"43c",
        vec![
            (1, LiteralDiag::Number(NumberDiag::StartsWithNumber)),
            (4, LiteralDiag::String(StringDiag::EndsWithQuote)),
        ],
    );
}
