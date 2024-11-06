use crate::lexer::{
    check, check_none,
    items::{item::ident::Ident, shared::distribution::Distribution},
    Parse, Slicable, Slice,
};
use macros::Parse;
use std::fmt::{Debug, Display};

#[derive(PartialEq, Debug, Parse)]
#[grammar(
    Ident Distribution
)]
pub struct BlockDistruct {
    name: Ident,
    dist: Distribution,
}

impl Display for BlockDistruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NamedDistruct({})", self.name)
    }
}

#[test]
fn parse_named_distruct() {
    check("main..", |code| BlockDistruct {
        name: Ident::new([0, 3], code),
        dist: Distribution(Slice::new([4, 5], code)),
    });
    check(" main  ..  ", |code| BlockDistruct {
        name: Ident::new([1, 4], code),
        dist: Distribution(Slice::new([7, 8], code)),
    });

    // error
    check_none::<BlockDistruct>(" main  .  ");
    check_none::<BlockDistruct>("..  ");
    check_none::<BlockDistruct>(" dsf. ");

    check_none::<BlockDistruct>(" ");
    check_none::<BlockDistruct>("");
}
