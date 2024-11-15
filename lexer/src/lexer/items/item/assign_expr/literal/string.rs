use macros::Slicable;

use crate::{
    lexer::{check, check_none, items::shared::whitespaces::Whitespaces, Code, Parse, Slice},
    Slicable,
};

#[derive(Debug, PartialEq, Slicable)]
pub struct String<'s>(pub Slice<'s>);

impl<'s> Parse<'s> for String<'s> {
    fn parse(code: &Code<'s>) -> Option<Self> {
        let code = &mut code.clone();

        Whitespaces::parse_and_consume(code);
        let mut iter = code.iter();

        let (i, char) = iter.next()?;
        let start = (char == '"').then_some(i)?;

        for (i, char) in iter {
            if char == '"' {
                return Some(Self(Slice::new(start..=i, code)));
            }
        }
        None
    }
}

#[test]
fn parse_string() {
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
