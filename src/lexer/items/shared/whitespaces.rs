use crate::lexer::{check, check_none, Code, Parse, Slicable, Slice, IGNORE};
use std_reset::prelude::Deref;

#[derive(PartialEq, Clone, Debug, Deref)]
pub struct Whitespaces(Slice);

impl Parse for Whitespaces {
    fn parse(code: &Code) -> Option<Self> {
        let mut end = None;
        for (i, char) in code.iter() {
            if IGNORE.contains(&char) {
                if i == code.len() - 1 {
                    end = Some(i)
                }
                continue;
            } else {
                if i == code.cursor {
                    return None;
                }
                end = Some(i - 1);
                break;
            }
        }

        Some(Self(Slice::new([code.cursor, end?], code)))
    }
}

impl Slicable for Whitespaces {
    fn get_start(&self) -> usize {
        self.0.get_start()
    }
    fn get_end(&self) -> usize {
        self.0.get_end()
    }
}

#[test]
fn parse_whitespace() {
    check(" ", |code| Whitespaces(Slice::new([0, 0], code)));
    check("     ", |code| Whitespaces(Slice::new([0, 4], code)));
    check("     f", |code| Whitespaces(Slice::new([0, 4], code)));
    check_none::<Whitespaces>("f     f");
    check_none::<Whitespaces>("f");
    check_none::<Whitespaces>("");
}
