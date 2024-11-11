mod number;
mod string;

use crate::{
    lexer::{Code, Parse, Slicable, Slice},
    parse_variants,
};
use number::parse_number;
use std::fmt::Display;
use string::parse_string;

#[derive(PartialEq, Debug, Clone, Hash, Eq)]
pub struct Literal {
    pub type_: LiteralType,
    pub slice: Slice,
}

#[derive(PartialEq, Debug, Clone, Hash, Eq)]
pub enum LiteralType {
    Number,
    String,
}

impl Literal {
    pub fn new(type_: LiteralType, slice: [usize; 2], code: &Code) -> Self {
        Self {
            type_,
            slice: Slice::new(slice, code),
        }
    }
}

impl Parse for Literal {
    fn parse(code: &Code) -> Option<Self> {
        parse_variants!(
            parse_number(code).map(|slice| Self {
                type_: LiteralType::Number,
                slice,
            }),
            parse_string(code).map(|slice| Self {
                type_: LiteralType::String,
                slice,
            })
        )
    }
}

impl Slicable for Literal {
    fn get_start(&self) -> usize {
        self.slice.get_start()
    }
    fn get_end(&self) -> usize {
        self.slice.get_end()
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Literal({:?}({}))", self.type_, self.slice)
    }
}

#[test]
fn parse_literal() {
    let check = |a, b: (LiteralType, [usize; 2])| {
        let code = &mut Code::new(a);
        assert_eq!(
            Literal::parse(code),
            Some(Literal {
                type_: b.0,
                slice: Slice::new(b.1, code)
            })
        );
    };
    let check_none = |a| {
        assert_eq!(Literal::parse(&mut Code::new(a)), None);
    };

    check("2", (LiteralType::Number, [0, 0]));
    check("2 ", (LiteralType::Number, [0, 0]));
    check(" 2", (LiteralType::Number, [1, 1]));
    check("  2  ", (LiteralType::Number, [2, 2]));
    check("  233", (LiteralType::Number, [2, 4]));
    check("  443  ", (LiteralType::Number, [2, 4]));
    check("3434", (LiteralType::Number, [0, 3]));

    check(r#""abc""#, (LiteralType::String, [0, 4]));
    check(r#""abc"  "#, (LiteralType::String, [0, 4]));
    check(r#"  "abc"  "#, (LiteralType::String, [2, 6]));
    check(r#"  " ab s3fsf d2_c "  "#, (LiteralType::String, [2, 18]));

    // errors
    check_none(" 2sdf ");
    check_none(" \"2sdf ");
}
