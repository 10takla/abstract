use super::whitespaces::Whitespaces;
use crate::{
    lexer::{check, check_none, Code, DiagParse, Diags, Slicable, Slice},
    Parse, Recognized,
};
use macros::Slicable;
use std_reset::prelude::Deref;

#[derive(PartialEq, Clone, Debug, Deref, Hash, Eq, Slicable)]
pub struct Distribution<'s>(pub Slice<'s>);

impl<'s> Parse<'s> for Distribution<'s> {
    type Diag = DistributionDiag;
    fn parse(
        code: &Code<'s>,
        diags: &mut Diags<Self::Diag>,
        recognized: &mut Recognized<'s>,
    ) -> Option<Self> {
        let code = &mut code.clone();
        Whitespaces::diag_and_consume(code, recognized);

        let start = code.cursor;
        matches!(code.get_offset_slice(2), Some("..")).then(|| {
            code.offset(2);
        })?;
        Some(Self(Slice::new(start..=code.cursor - 1, code)))
    }
}

impl<'s> DiagParse<'s> for Distribution<'s> {}

#[derive(PartialEq, Debug, Clone)]
pub enum DistributionDiag {}

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
