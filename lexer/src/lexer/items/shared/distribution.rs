use super::whitespaces::Whitespaces;
use crate::lexer::{check, check_none, Code, DiagParse, Diags, Slicable, Slice};
use macros::Slicable;
use std_reset::prelude::Deref;

#[derive(PartialEq, Clone, Debug, Deref, Hash, Eq, Slicable)]
pub struct Distribution<'s>(pub Slice<'s>);

impl<'s> DiagParse<'s> for Distribution<'s> {
    type Diag = DistributionDiag;
    fn parse(code: &Code<'s>, diags: &mut Diags<Self::Diag>) -> Option<Self> {
        let code = &mut code.clone();
        Whitespaces::parse_and_consume(code, &mut vec![]);

        let start = code.cursor;
        matches!(code.get_offset_slice(2), Some("..")).then(|| {
            code.offset(2);
        })?;
        Some(Self(Slice::new(start..=code.cursor - 1, code)))
    }
}

#[derive(PartialEq, Debug)]
pub enum DistributionDiag {
}

#[test]
fn parse() {
    check(" .. ", |code| Distribution(Slice::new(1..=2, code)));
    check(" ..", |code| Distribution(Slice::new(1..=2, code)));
    check("..", |code| Distribution(Slice::new(0..=1, code)));

    // errors
    check_none::<Distribution>(".");
    check_none::<Distribution>("");
    check_none::<Distribution>("  ");
    check_none::<Distribution>(" . ");
    check_none::<Distribution>(" .");
}
