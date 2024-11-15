use crate::lexer::{items::shared::whitespaces::Whitespaces, Code, Parse, Slice};

pub fn parse_string(code: &Code) -> Option<Slice> {
    let code = &mut code.clone();

    Whitespaces::parse_and_consume(code);
    let mut iter = code.iter();

    let (i, char) = iter.next()?;
    let start = (char == '"').then_some(i)?;

    for (i, char) in iter {
        if char == '"' {
            return Some(Slice::new(start..=i, code));
        }
    }
    None
}

#[test]
fn parse_string_test() {
    let check = |a, b: fn(&Code) -> Slice| {
        let code = &mut Code::new(a);
        assert_eq!(parse_string(code), Some(b(code)));
    };
    let check_none = |a| {
        assert_eq!(parse_string(&mut Code::new(a)), None);
    };

    check(r#""abc""#, |code| Slice::new(0..=4, code));
    check(r#""abc"  "#, |code| Slice::new(0..=4, code));
    check(r#"  "abc"  "#, |code| Slice::new(2..=6, code));
    check(r#"  " ab s3fsf d2_c "  "#, |code| Slice::new(2..=18, code));
    check(r#""""#, |code| Slice::new(0..=1, code));
    check(r#" " " "#, |code| Slice::new(1..=3, code));

    // errors
    check_none(" 2sdf ");
    check_none(" \"2sdf ");
    check_none(" \" ");
}
