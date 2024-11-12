use super::whitespaces::Whitespaces;
use crate::lexer::{check, check_none, Code, Parse, Slicable, Slice};
use std_reset::prelude::Deref;

#[derive(PartialEq, Clone, Debug, Deref, Hash, Eq)]
pub struct Distribution(pub Slice);

impl Parse for Distribution {
    fn parse(code: &Code) -> Option<Self> {
        let code = &mut code.clone();
        Whitespaces::parse_and_consume(code);

        let start = code.cursor;
        matches!(code.get_slice(2), Some("..")).then(|| {
            code.offset(2);
        })?;
        Some(Self(Slice::new([start, code.cursor - 1], code)))
    }
}

impl Slicable for Distribution {
    fn get_start(&self) -> usize {
        self.0.get_start()
    }
    fn get_end(&self) -> usize {
        self.0.get_end()
    }
}

#[test]
fn parse_distribution() {
    check(" .. ", |code| Distribution(Slice::new([1, 2], code)));
    check(" ..", |code| Distribution(Slice::new([1, 2], code)));
    check("..", |code| Distribution(Slice::new([0, 1], code)));

    // errors
    check_none::<Distribution>(".");
    check_none::<Distribution>("");
    check_none::<Distribution>("  ");
    check_none::<Distribution>(" . ");
    check_none::<Distribution>(" .");
}
