use crate::{lexer::{
    check, check_none, items::{item::ident::Ident, shared::distribution::Distribution}, Parse, Slicable
}, Slice};
use macros::Parse;
use std::fmt::{Debug, Display};

#[derive(PartialEq, Debug, Parse, Hash, Eq, Clone)]
#[grammar(
    Ident Distribution
)]
pub struct CallBlockDistruct<'s> {
    pub name: Ident<'s>,
    pub dist: Distribution<'s>,
}

impl Display for CallBlockDistruct<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NamedDistruct({})", self.name)
    }
}

#[test]
fn parse_named_distruct() {
    check("main..", |code| CallBlockDistruct {
        name: Ident::new(0..=3, code),
        dist: Distribution(Slice::new(4..=5, code)),
    });
    check(" main  ..  ", |code| CallBlockDistruct {
        name: Ident::new(1..=4, code),
        dist: Distribution(Slice::new(7..=8, code)),
    });

    // error
    check_none::<CallBlockDistruct>(" main  .  ");
    check_none::<CallBlockDistruct>("..  ");
    check_none::<CallBlockDistruct>(" dsf. ");

    check_none::<CallBlockDistruct>(" ");
    check_none::<CallBlockDistruct>("");
}
