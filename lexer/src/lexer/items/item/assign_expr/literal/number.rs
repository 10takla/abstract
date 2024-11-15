use crate::lexer::{items::shared::whitespaces::Whitespaces, Code, Parse, Slice, IGNORE};

pub fn parse_number(code: &Code) -> Option<Slice> {
    let code = &mut code.clone();

    Whitespaces::parse_and_consume(code);
    let mut iter = code.iter();

    let (i, char) = iter.next()?;
    let start = if char.is_digit(10) {
        if i == code.len() - 1 {
            return Some(Slice::new(i..=i, code));
        }
        i
    } else {
        return None;
    };

    let t = || {
        for (i, char) in iter {
            if IGNORE.contains(&char) {
                return Some(i - 1);
            }
            if char.is_digit(10) {
                if i == code.len() - 1 {
                    return Some(i);
                }
                continue;
            }
            return None;
        }
        None
    };
    let end = t()?;

    Some(Slice::new(start..=end, code))
}

#[test]
fn parse_number_test() {
    let check = |a, b| {
        let code = &mut Code::new(a);
        assert_eq!(parse_number(code), Some(Slice::new(b, code)));
    };
    let check_none = |a| {
        assert_eq!(parse_number(&mut Code::new(a)), None);
    };

    check("2", 0..=0);
    check("2 ", 0..=0);
    check(" 2", 1..=1);
    check("  2  ", 2..=2);
    check("  233", 2..=4);
    check("  443  ", 2..=4);
    check("3434", 0..=3);

    // errors
    check_none("");
    check_none("  ");
    check_none("  2sdf ");
    check_none("  abc  ");
    check_none("abc123");
}
