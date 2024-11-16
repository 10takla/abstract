pub mod number;
pub mod string;

use crate::{
    lexer::{check, check_none, Code, Parse, Slicable, Slice},
    parse_variants,
};
use std::{fmt::Display, ops::RangeInclusive};
use macros::Slicable;
use number::Number;
use string::String;

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

impl<'s> Literal<'s> {
    pub fn new(type_: LiteralType, slice: RangeInclusive<usize>, code: &Code<'s>) -> Self {
        Self {
            type_,
            slice: Slice::new(slice, code),
        }
    }
}

impl<'s> Parse<'s> for Literal<'s> {
    fn parse(code: &Code<'s>) -> Option<Self> {
        parse_variants!(
            Number::parse(code).map(|v| Self {
                type_: LiteralType::Number,
                slice: v.0,
            }),
            String::parse(code).map(|v| Self {
                type_: LiteralType::String,
                slice: v.0,
            })
        )
    }
}

impl Display for Literal<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Literal({:?}({}))", self.type_, self.slice)
    }
}

#[test]
fn parse_literal() {
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
