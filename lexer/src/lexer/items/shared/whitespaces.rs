use crate::{
    lexer::{check, check_none, Code, DiagParse, Diags, Slicable, Slice, IGNORE},
    Parse, Recognized,
};
use macros::Slicable;
use std_reset::prelude::Deref;

#[derive(PartialEq, Clone, Debug, Deref, Slicable)]
pub struct Whitespaces<'s>(Slice<'s>);

#[derive(Clone, Debug)]
pub enum WhitespacesDiag {}

impl<'s> Parse<'s> for Whitespaces<'s> {
    type Diag = WhitespacesDiag;
    fn parse(
        code: &Code<'s>,
        diags: &mut Diags<WhitespacesDiag>,
        recognized: &mut Recognized<'s>,
    ) -> Option<Self> {
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

        Some(Self(Slice::new(code.cursor..=end?, code)))
    }
}

impl<'s> DiagParse<'s> for Whitespaces<'s> {}

#[test]
fn parse() {
    check(" ", |code| Whitespaces(Slice::new(0..=0, code)));
    check("     ", |code| Whitespaces(Slice::new(0..=4, code)));
    check("     f", |code| Whitespaces(Slice::new(0..=4, code)));
    check_none::<Whitespaces>("f     f");
    check_none::<Whitespaces>("f");
    check_none::<Whitespaces>("");
}
