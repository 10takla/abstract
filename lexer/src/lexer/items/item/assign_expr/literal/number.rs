use crate::{
    lexer::{
        check, check_none, items::shared::whitespaces::Whitespaces, Code, Parse, Slice, IGNORE,
    },
    Slicable,
};

#[derive(PartialEq, Debug)]
pub struct Number<'s>(pub Slice<'s>);

impl<'s> Parse<'s> for Number<'s> {
    fn parse(code: &Code<'s>) -> Option<Self> {
        let code = &mut code.clone();

        Whitespaces::parse_and_consume(code);
        let mut iter = code.iter();

        let (i, char) = iter.next()?;
        let start = if char.is_digit(10) {
            if i == code.len() - 1 {
                return Some(Self(Slice::new(i..=i, code)));
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

        Some(Self(Slice::new(start..=end, code)))
    }
}

impl Slicable for Number<'_> {
    fn get_start(&self) -> usize {
        self.0.get_start()
    }
    fn get_end(&self) -> usize {
        self.0.get_end()
    }
}

#[test]
fn parse_number() {
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
