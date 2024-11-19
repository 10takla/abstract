use crate::{
    items::{
        item::{block::init::named::NamedBlockDiag, ident::IdentDiag},
        shared::distribution::DistributionDiag,
    },
    lexer::{
        check, check_none,
        items::{item::ident::Ident, shared::distribution::Distribution},
        DiagParse, Slicable,
    },
    Slice,
};
use macros::Parse;
use std::fmt::{Debug, Display};

#[derive(PartialEq, Debug, Parse, Hash, Eq, Clone)]
#[grammar(
    Ident Distribution
)]
#[diag(CallBlockDistructDiag)]
pub struct CallBlockDistruct<'s> {
    #[diag(Name)]
    pub name: Ident<'s>,
    #[diag(Distribution)]
    pub dist: Distribution<'s>,
}

#[derive(PartialEq, Debug)]
pub enum CallBlockDistructDiag {
    Name(IdentDiag),
    Distribution(DistributionDiag),
}

impl Display for CallBlockDistruct<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NamedDistruct({})", self.name)
    }
}

#[test]
fn parse() {
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
